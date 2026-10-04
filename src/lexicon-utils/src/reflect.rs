// Scoped reflection (Spec §30).
//
// Deliberately bounded: type/field/method inspection, struct tags,
// serialization hooks and a dynamic function registry. Reflection can
// read private fields ONLY through an explicit `allow_private` grant,
// can never invoke `unsafe`-marked functions, and never retains
// references beyond the registry lifetime.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
    /// Struct tags, e.g. `json:"id"`.
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<String>,
}

impl TypeInfo {
    pub fn field(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn tag(&self, field: &str, key: &str) -> Option<&str> {
        self.field(field)?.tags.get(key).map(|s| s.as_str())
    }

    // ---- Part X / §30: field + method inspection helpers ----
    /// Field names in declaration order (Spec §30 field inspection).
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|f| f.name.as_str()).collect()
    }

    /// Method names in declaration order (Spec §30 method inspection).
    pub fn method_names(&self) -> Vec<&str> {
        self.methods.iter().map(|m| m.as_str()).collect()
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.field(name).is_some()
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.methods.iter().any(|m| m == name)
    }

    /// Fields carrying `key = value` in their struct tags.
    pub fn fields_with_tag(&self, key: &str, value: &str) -> Vec<&FieldInfo> {
        self.fields
            .iter()
            .filter(|f| f.tags.get(key).map(|v| v == value).unwrap_or(false))
            .collect()
    }
}

/// Runtime value snapshot for value inspection (Spec §30).
///
/// `ValueInfo` is a deliberate projection: it carries owned data (no
/// borrows retained beyond the call) so reflection can never extend a
/// value lifetime. Construct via [`TypeRegistry::describe_value`].
#[derive(Debug, Clone, PartialEq)]
pub enum ValueInfo {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ValueInfo>),
    Object { type_name: String, fields: Vec<(String, ValueInfo)> },
    Option(Option<Box<ValueInfo>>),
    Result(std::result::Result<Box<ValueInfo>, String>),
}

impl ValueInfo {
    /// Type name of this value snapshot (RTTI, Spec §30).
    pub fn type_name(&self) -> &'static str {
        match self {
            ValueInfo::Null => "null",
            ValueInfo::Bool(_) => "bool",
            ValueInfo::Int(_) => "int",
            ValueInfo::Float(_) => "float",
            ValueInfo::String(_) => "String",
            ValueInfo::Array(_) => "array",
            ValueInfo::Object { .. } => "object",
            ValueInfo::Option(_) => "Option",
            ValueInfo::Result(_) => "Result",
        }
    }

    pub fn is_null_or_none(&self) -> bool {
        matches!(self, ValueInfo::Null | ValueInfo::Option(None))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, ValueInfo::Result(Ok(_)))
    }

    /// Field lookup on object snapshots (value inspection).
    pub fn field(&self, name: &str) -> Option<&ValueInfo> {
        match self {
            ValueInfo::Object { fields, .. } => {
                fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
            }
            _ => None,
        }
    }
}

type DynFn = Box<dyn Fn(Vec<String>) -> Result<String, String> + Send + Sync>;

struct FnEntry {
    func: DynFn,
    /// Whether this function touches unsafe operations (never invocable).
    is_unsafe: bool,
    visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

pub struct TypeRegistry {
    types: HashMap<String, TypeInfo>,
    fns: HashMap<String, FnEntry>,
    allow_private: bool,
}

impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry { types: HashMap::new(), fns: HashMap::new(), allow_private: false }
    }

    /// Grant read access to private fields/methods for tooling (debuggers,
    /// serializers). Off by default.
    pub fn set_allow_private(&mut self, allow: bool) {
        self.allow_private = allow;
    }

    pub fn register_type(&mut self, info: TypeInfo) {
        self.types.insert(info.name.clone(), info);
    }

    pub fn inspect(&self, name: &str) -> Option<&TypeInfo> {
        self.types.get(name)
    }

    pub fn type_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.types.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn register_fn(
        &mut self,
        name: &str,
        visibility: Visibility,
        is_unsafe: bool,
        func: DynFn,
    ) {
        self.fns.insert(name.to_string(), FnEntry { func, is_unsafe, visibility });
    }

    /// Dynamic invocation (Spec §30): refuses unsafe functions and
    /// private functions unless explicitly granted.
    pub fn invoke(&self, name: &str, args: Vec<String>) -> Result<String, String> {
        let entry = self.fns.get(name)
            .ok_or_else(|| format!("E0302: unknown function `{}`", name))?;
        if entry.is_unsafe {
            return Err(format!("E0305: reflection cannot invoke unsafe function `{}`", name));
        }
        if entry.visibility == Visibility::Private && !self.allow_private {
            return Err(format!("E0305: private function `{}` not visible to reflection", name));
        }
        (entry.func)(args)
    }

    /// Serialization hook: project a field map using `json` tags,
    /// falling back to field names.
    pub fn to_tagged_map(
        &self,
        type_name: &str,
        values: &HashMap<String, String>,
    ) -> Option<HashMap<String, String>> {
        let info = self.types.get(type_name)?;
        let mut out = HashMap::new();
        for f in &info.fields {
            if let Some(v) = values.get(&f.name) {
                let key = f.tags.get("json").cloned().unwrap_or_else(|| f.name.clone());
                out.insert(key, v.clone());
            }
        }
        Some(out)
    }
}

impl TypeRegistry {
    /// Build a [`ValueInfo`] snapshot for a tagged field map
    /// (`type_name` + `field -> string value`). Unknown fields are kept
    /// as strings; the snapshot owns everything (no lifetime capture).
    pub fn describe_value(
        &self,
        type_name: &str,
        values: &HashMap<String, String>,
    ) -> Option<ValueInfo> {
        let info = self.types.get(type_name)?;
        let fields = info
            .fields
            .iter()
            .filter_map(|f| {
                values
                    .get(&f.name)
                    .map(|v| (f.name.clone(), ValueInfo::String(v.clone())))
            })
            .collect();
        Some(ValueInfo::Object {
            type_name: type_name.to_string(),
            fields,
        })
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Part X / §35–§36: metaprogramming — `MacroExpander` stub + source mapping.
//
// Contract (Spec §35): macro expansion MUST preserve source mapping so
// diagnostics/debug can point at the original call site. This stub
// performs deterministic `$name` textual substitution over registered
// templates (no host I/O, no filesystem, no network) and returns the
// expanded text together with a [`SourceMapping`]. Procedural/host
// plugins are out of scope for the stub and must go through the
// sandboxed plugin boundary (`rt.rs`).
// ---------------------------------------------------------------------------

/// Original call-site position preserved through expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl SourcePos {
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        SourcePos { file: file.into(), line, column }
    }
}

/// Mapping from an expanded range back to its macro call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapping {
    pub macro_name: String,
    pub call_site: SourcePos,
    /// `(expanded_offset, original_arg_index)` anchors. Empty when the
    /// expansion contains no substituted arguments.
    pub anchors: Vec<(usize, usize)>,
}

impl SourceMapping {
    /// Look up which original argument produced `expanded_offset`
    /// (nearest anchor at or before the offset).
    pub fn arg_at(&self, expanded_offset: usize) -> Option<usize> {
        let mut best = None;
        for (off, arg) in &self.anchors {
            if *off <= expanded_offset {
                best = Some(*arg);
            } else {
                break;
            }
        }
        best
    }
}

/// Deterministic macro-template expander (stub).
///
/// Templates use `$0`, `$1`, … placeholders for positional arguments.
/// Expansion is pure string substitution; unknown macros and arity
/// mismatches are explicit `Err` diagnostics (no panics).
#[derive(Debug, Default)]
pub struct MacroExpander {
    templates: HashMap<String, String>,
}

impl MacroExpander {
    pub fn new() -> Self {
        MacroExpander { templates: HashMap::new() }
    }

    /// Register a template (e.g. `define("vec2", "($0, $1)")`).
    pub fn define(&mut self, name: &str, template: &str) {
        self.templates.insert(name.to_string(), template.to_string());
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// Expand `name` with `args` at `call_site`, returning the expanded
    /// text plus its source mapping.
    pub fn expand(
        &self,
        name: &str,
        args: &[String],
        call_site: SourcePos,
    ) -> std::result::Result<(String, SourceMapping), String> {
        let template = self.templates.get(name).ok_or_else(|| {
            format!("E0201: unknown macro `{}`", name)
        })?;
        let mut out = String::new();
        let mut anchors = Vec::new();
        let bytes = template.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'$' {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j == i + 1 {
                    return Err(format!(
                        "E0201: stray `$` in macro `{}` (use `$0`, `$1`, …)",
                        name
                    ));
                }
                let idx: usize = template[i + 1..j].parse().map_err(|_| {
                    format!("E0201: bad placeholder in macro `{}`", name)
                })?;
                let arg = args.get(idx).ok_or_else(|| {
                    format!(
                        "E0201: macro `{}` expects argument ${} (got {} args)",
                        name,
                        idx,
                        args.len()
                    )
                })?;
                anchors.push((out.len(), idx));
                out.push_str(arg);
                i = j;
            } else {
                out.push(bytes[i] as char);
                i += 1;
            }
        }
        let mapping = SourceMapping { macro_name: name.to_string(), call_site, anchors };
        Ok((out, mapping))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TypeRegistry {
        let mut r = TypeRegistry::new();
        let mut tags = HashMap::new();
        tags.insert("json".to_string(), "id".to_string());
        r.register_type(TypeInfo {
            name: "User".to_string(),
            fields: vec![
                FieldInfo { name: "user_id".to_string(), ty: "int".to_string(), tags },
                FieldInfo { name: "name".to_string(), ty: "String".to_string(), tags: HashMap::new() },
            ],
            methods: vec!["greet".to_string()],
        });
        r.register_fn("greet", Visibility::Public, false, Box::new(|args| {
            Ok(format!("hi {}", args.first().cloned().unwrap_or_default()))
        }));
        r.register_fn("raw", Visibility::Public, true, Box::new(|_| Ok(String::new())));
        r.register_fn("secret", Visibility::Private, false, Box::new(|_| Ok("s".to_string())));
        r
    }

    #[test]
    fn inspect_and_tags() {
        let r = sample();
        let t = r.inspect("User").unwrap();
        assert_eq!(t.tag("user_id", "json"), Some("id"));
        assert_eq!(t.tag("name", "json"), None);
        assert_eq!(r.type_names(), vec!["User".to_string()]);
    }

    #[test]
    fn invoke_rules() {
        let mut r = sample();
        assert_eq!(r.invoke("greet", vec!["bob".to_string()]).unwrap(), "hi bob");
        assert!(r.invoke("raw", vec![]).is_err());
        assert!(r.invoke("secret", vec![]).is_err());
        assert!(r.invoke("missing", vec![]).is_err());
        r.set_allow_private(true);
        assert!(r.invoke("secret", vec![]).is_ok());
    }

    #[test]
    fn serialization_hook() {
        let r = sample();
        let mut values = HashMap::new();
        values.insert("user_id".to_string(), "7".to_string());
        values.insert("name".to_string(), "ana".to_string());
        let map = r.to_tagged_map("User", &values).unwrap();
        assert_eq!(map.get("id"), Some(&"7".to_string()));
        assert_eq!(map.get("name"), Some(&"ana".to_string()));
    }

    #[test]
    fn inspection_helpers_and_value_info() {
        let r = sample();
        let t = r.inspect("User").unwrap();
        assert_eq!(t.field_names(), vec!["user_id", "name"]);
        assert_eq!(t.method_names(), vec!["greet"]);
        assert!(t.has_field("name") && !t.has_field("missing"));
        assert!(t.has_method("greet") && !t.has_method("nope"));
        assert_eq!(t.fields_with_tag("json", "id").len(), 1);
        let mut values = HashMap::new();
        values.insert("name".to_string(), "ana".to_string());
        let v = r.describe_value("User", &values).unwrap();
        assert_eq!(v.type_name(), "object");
        assert_eq!(v.field("name"), Some(&ValueInfo::String("ana".to_string())));
        assert!(r.describe_value("Nope", &values).is_none());
        assert!(ValueInfo::Option(None).is_null_or_none());
        assert!(ValueInfo::Result(Ok(Box::new(ValueInfo::Int(1)))).is_ok());
    }

    #[test]
    fn macro_expander_preserves_source_mapping() {
        let mut m = MacroExpander::new();
        m.define("pair", "($0, $1)");
        assert!(m.is_defined("pair"));
        let site = SourcePos::new("f.lex", 3, 5);
        let (text, mapping) = m.expand("pair", &["a".to_string(), "b".to_string()], site.clone()).unwrap();
        assert_eq!(text, "(a, b)");
        assert_eq!(mapping.macro_name, "pair");
        assert_eq!(mapping.call_site, site);
        assert_eq!(mapping.arg_at(1), Some(0));
        assert!(m.expand("missing", &[], SourcePos::new("f", 0, 0)).is_err());
        assert!(m.expand("pair", &["only".to_string()], SourcePos::new("f", 0, 0)).is_err());
    }
}
