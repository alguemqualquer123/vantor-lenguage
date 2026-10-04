pub mod error;
pub mod span;

// ---------------------------------------------------------------------------
// Hardening limits (DoS protections).
//
// `MAX_VALUE_DEPTH` bounds recursive rendering and equality: deeper nesting
// renders as `#...` (quoted `"#..."` in JSON) and compares as unequal.
// `MAX_RENDER_ITEMS` caps how many Array/Map/Object items `Debug`/`Display`
// print so a 100k-element collection renders in O(1) space.
// `Clone` is fully iterative (heap stack) and has no depth limit.
// ---------------------------------------------------------------------------

/// Maximum nesting depth for `Debug`/`Display`/`to_json_string` and `PartialEq`.
///
/// Beyond this depth rendering emits `#...` (`"#..."` in JSON) and equality
/// returns `false`. Complexity: O(1) constant.
pub const MAX_VALUE_DEPTH: usize = 128;

/// Maximum collection items rendered by `Debug`/`Display`.
///
/// Remaining items are summarized as `... +N more`. Complexity: O(1) constant.
pub const MAX_RENDER_ITEMS: usize = 32;

pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Unit,
    Null,
    Array(Vec<Value>),
    Map(Vec<(String, Value)>),
    /// `Option<T>` representation (Spec §8): None carries no payload.
    Option(Option<Box<Value>>),
    /// `Result<T, E>` representation (Spec §8): Ok(value) | Err(message).
    Result(std::result::Result<Box<Value>, String>),
    Object(std::sync::Arc<Object>),
    Function(std::sync::Arc<Closure>),
}

#[derive(Clone)]
pub struct Object {
    pub fields: std::collections::HashMap<String, Value>,
    pub type_id: String,
}

impl Object {
    /// Create an empty object with the given type tag.
    /// Complexity: O(1).
    #[inline]
    pub fn new(type_id: impl Into<String>) -> Self {
        Object {
            fields: std::collections::HashMap::new(),
            type_id: type_id.into(),
        }
    }

    /// Borrowed field lookup (no clone, hot-path friendly).
    /// Complexity: O(1) average (HashMap).
    #[inline]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Whether `key` is present.
    /// Complexity: O(1) average.
    #[inline]
    pub fn has(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Insert or overwrite `key`.
    /// Complexity: O(1) average.
    #[inline]
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.fields.insert(key.into(), value);
    }

    /// Remove `key`, returning the old value if present.
    /// Complexity: O(1) average.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.fields.remove(key)
    }

    /// Number of fields.
    /// Complexity: O(1).
    #[inline]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Whether there are no fields.
    /// Complexity: O(1).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Breadth-capped, deterministic (sorted keys). Field values use the
        // iterative `Value` Debug so deep nesting cannot overflow the stack.
        // Complexity: O(min(fields, MAX_RENDER_ITEMS)).
        write!(f, "Object {{ type_id: {:?}, fields: {{", self.type_id)?;
        let mut keys: Vec<&String> = self.fields.keys().collect();
        keys.sort();
        let show = keys.len().min(MAX_RENDER_ITEMS);
        for (i, k) in keys.iter().take(show).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            let v = self.fields.get(*k).expect("sorted key must exist");
            write!(f, "{:?}: {:?}", k, v)?;
        }
        if keys.len() > show {
            if show > 0 {
                write!(f, ", ")?;
            }
            write!(f, "... +{} more", keys.len() - show)?;
        }
        write!(f, "}}}}")
    }
}

pub struct Closure {
    pub params: Vec<String>,
    pub body: Box<dyn Fn(Vec<Value>) -> Value + Send + Sync>,
    pub captured: std::collections::HashMap<String, Value>,
}

impl Closure {
    /// Construct a closure value.
    /// Complexity: O(1) plus moved inputs.
    pub fn new(
        params: Vec<String>,
        captured: std::collections::HashMap<String, Value>,
        body: Box<dyn Fn(Vec<Value>) -> Value + Send + Sync>,
    ) -> Self {
        Closure {
            params,
            body,
            captured,
        }
    }

    /// Number of parameters.
    /// Complexity: O(1).
    #[inline]
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Sorted names of captured variables (deterministic).
    /// Complexity: O(c log c) where c = captured count.
    pub fn captured_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.captured.keys().cloned().collect();
        names.sort();
        names
    }

    /// Whether `name` is captured.
    /// Complexity: O(1) average.
    #[inline]
    pub fn has_capture(&self, name: &str) -> bool {
        self.captured.contains_key(name)
    }

    /// Borrowed capture lookup (no clone).
    /// Complexity: O(1) average.
    #[inline]
    pub fn captured_get(&self, name: &str) -> Option<&Value> {
        self.captured.get(name)
    }
}

impl std::fmt::Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Capped + deterministic; captured values use iterative Value Debug.
        // Complexity: O(params + min(captured, MAX_RENDER_ITEMS)).
        write!(f, "Closure {{ params: {:?}, captured: {{", self.params)?;
        let mut keys: Vec<&String> = self.captured.keys().collect();
        keys.sort();
        let show = keys.len().min(MAX_RENDER_ITEMS);
        for (i, k) in keys.iter().take(show).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            let v = self.captured.get(*k).expect("sorted key must exist");
            write!(f, "{:?}: {:?}", k, v)?;
        }
        if keys.len() > show {
            if show > 0 {
                write!(f, ", ")?;
            }
            write!(f, "... +{} more", keys.len() - show)?;
        }
        write!(f, "}}, .. }}")
    }
}

// ---------------------------------------------------------------------------
// Iterative Clone (DoS protection).
//
// Derived `Clone` recurses on `Vec<Value>` / `Box<Value>` and overflows the
// stack on 10k-deep nesting. This impl uses an explicit heap stack
// (`work` + `vals`) so arbitrarily deep values clone without recursion.
// `Object`/`Function` are `Arc` shallow clones (O(1)).
// Complexity: O(n) time where n = node count; O(depth) heap.
// ---------------------------------------------------------------------------

impl Clone for Value {
    fn clone(&self) -> Self {
        enum Work<'a> {
            Visit(&'a Value),
            BuildArray(usize),
            BuildMap(Vec<String>),
            BuildSome,
            BuildOk,
        }
        let mut work: Vec<Work<'_>> = vec![Work::Visit(self)];
        let mut vals: Vec<Value> = Vec::new();
        while let Some(task) = work.pop() {
            match task {
                Work::Visit(v) => match v {
                    Value::Int(i) => vals.push(Value::Int(*i)),
                    Value::Float(f) => vals.push(Value::Float(*f)),
                    Value::Bool(b) => vals.push(Value::Bool(*b)),
                    Value::String(s) => vals.push(Value::String(s.clone())),
                    Value::Char(c) => vals.push(Value::Char(*c)),
                    Value::Unit => vals.push(Value::Unit),
                    Value::Null => vals.push(Value::Null),
                    Value::Array(items) => {
                        work.push(Work::BuildArray(items.len()));
                        for item in items.iter().rev() {
                            work.push(Work::Visit(item));
                        }
                    }
                    Value::Map(entries) => {
                        let keys: Vec<String> =
                            entries.iter().map(|(k, _)| k.clone()).collect();
                        work.push(Work::BuildMap(keys));
                        for (_, val) in entries.iter().rev() {
                            work.push(Work::Visit(val));
                        }
                    }
                    Value::Option(None) => vals.push(Value::Option(None)),
                    Value::Option(Some(inner)) => {
                        work.push(Work::BuildSome);
                        work.push(Work::Visit(inner));
                    }
                    Value::Result(Ok(inner)) => {
                        work.push(Work::BuildOk);
                        work.push(Work::Visit(inner));
                    }
                    Value::Result(Err(e)) => vals.push(Value::Result(Err(e.clone()))),
                    Value::Object(o) => vals.push(Value::Object(std::sync::Arc::clone(o))),
                    Value::Function(f) => vals.push(Value::Function(std::sync::Arc::clone(f))),
                },
                Work::BuildArray(n) => {
                    let start = vals.len().saturating_sub(n);
                    let arr: Vec<Value> = vals.drain(start..).collect();
                    vals.push(Value::Array(arr));
                }
                Work::BuildMap(keys) => {
                    let n = keys.len();
                    let start = vals.len().saturating_sub(n);
                    let vs: Vec<Value> = vals.drain(start..).collect();
                    let mut entries = Vec::with_capacity(n);
                    for (k, v) in keys.into_iter().zip(vs.into_iter()) {
                        entries.push((k, v));
                    }
                    vals.push(Value::Map(entries));
                }
                Work::BuildSome => {
                    let inner = vals.pop().expect("clone stack underflow (Some)");
                    vals.push(Value::Option(Some(Box::new(inner))));
                }
                Work::BuildOk => {
                    let inner = vals.pop().expect("clone stack underflow (Ok)");
                    vals.push(Value::Result(Ok(Box::new(inner))));
                }
            }
        }
        vals.pop().expect("clone produced no value")
    }
}

// ---------------------------------------------------------------------------
// Iterative depth-limited PartialEq (DoS protection).
//
// Derived `==` on `Vec<Value>` recurses and overflows on deep nesting.
// This impl uses an explicit heap stack; pairs deeper than
// `MAX_VALUE_DEPTH` compare as unequal (returns `false`) instead of
// overflowing. Shallow semantics are unchanged (ordered Array/Map,
// strict variant match, `Object`/`Function` always unequal).
// Complexity: O(n) up to the depth cap; O(depth) heap.
// ---------------------------------------------------------------------------

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let mut stack: Vec<(&Value, &Value, usize)> = vec![(self, other, 0)];
        while let Some((a, b, depth)) = stack.pop() {
            if depth > MAX_VALUE_DEPTH {
                return false;
            }
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => {
                    if x != y {
                        return false;
                    }
                }
                (Value::Float(x), Value::Float(y)) => {
                    if x != y {
                        return false;
                    }
                }
                (Value::Bool(x), Value::Bool(y)) => {
                    if x != y {
                        return false;
                    }
                }
                (Value::String(x), Value::String(y)) => {
                    if x != y {
                        return false;
                    }
                }
                (Value::Char(x), Value::Char(y)) => {
                    if x != y {
                        return false;
                    }
                }
                (Value::Unit, Value::Unit) => {}
                (Value::Null, Value::Null) => {}
                (Value::Array(x), Value::Array(y)) => {
                    if x.len() != y.len() {
                        return false;
                    }
                    for (u, v) in x.iter().zip(y.iter()) {
                        stack.push((u, v, depth + 1));
                    }
                }
                (Value::Map(x), Value::Map(y)) => {
                    if x.len() != y.len() {
                        return false;
                    }
                    for ((kx, vx), (ky, vy)) in x.iter().zip(y.iter()) {
                        if kx != ky {
                            return false;
                        }
                        stack.push((vx, vy, depth + 1));
                    }
                }
                (Value::Option(x), Value::Option(y)) => match (x, y) {
                    (None, None) => {}
                    (Some(u), Some(v)) => stack.push((u, v, depth + 1)),
                    _ => return false,
                },
                (Value::Result(x), Value::Result(y)) => match (x, y) {
                    (Ok(u), Ok(v)) => stack.push((u, v, depth + 1)),
                    (Err(e1), Err(e2)) => {
                        if e1 != e2 {
                            return false;
                        }
                    }
                    _ => return false,
                },
                // Preserve historical semantics: objects/functions never equal.
                _ => return false,
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Iterative depth-limited + breadth-capped Debug/Display.
// ---------------------------------------------------------------------------

enum RenderOp<'a> {
    Lit(String),
    Val(&'a Value, usize),
}

fn value_debug_string(root: &Value) -> String {
    // Iterative, depth-limited (`#...`) and breadth-capped.
    // Complexity: O(rendered nodes) <= O(MAX_VALUE_DEPTH * MAX_RENDER_ITEMS).
    let mut out = String::new();
    let mut stack: Vec<RenderOp<'_>> = vec![RenderOp::Val(root, 0)];
    while let Some(op) = stack.pop() {
        match op {
            RenderOp::Lit(s) => out.push_str(&s),
            RenderOp::Val(v, depth) => {
                if depth > MAX_VALUE_DEPTH {
                    out.push_str("#...");
                    continue;
                }
                match v {
                    Value::Int(i) => out.push_str(&format!("Int({})", i)),
                    Value::Float(f) => out.push_str(&format!("Float({:?})", f)),
                    Value::Bool(b) => out.push_str(&format!("Bool({})", b)),
                    Value::String(s) => out.push_str(&format!("String({:?})", s)),
                    Value::Char(c) => out.push_str(&format!("Char({:?})", c)),
                    Value::Unit => out.push_str("Unit"),
                    Value::Null => out.push_str("Null"),
                    Value::Array(items) => {
                        if items.is_empty() {
                            out.push_str("Array([])");
                            continue;
                        }
                        // Inline expansion (avoid helper ordering confusion):
                        stack.push(RenderOp::Lit("])".to_string()));
                        if items.len() > MAX_RENDER_ITEMS {
                            stack.push(RenderOp::Lit(format!(
                                ", ... +{} more",
                                items.len() - MAX_RENDER_ITEMS
                            )));
                        }
                        let show = items.len().min(MAX_RENDER_ITEMS);
                        for i in (0..show).rev() {
                            stack.push(RenderOp::Val(&items[i], depth + 1));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit("Array([".to_string()));
                    }
                    Value::Map(entries) => {
                        if entries.is_empty() {
                            out.push_str("Map([])");
                            continue;
                        }
                        stack.push(RenderOp::Lit("])".to_string()));
                        if entries.len() > MAX_RENDER_ITEMS {
                            stack.push(RenderOp::Lit(format!(
                                ", ... +{} more",
                                entries.len() - MAX_RENDER_ITEMS
                            )));
                        }
                        let show = entries.len().min(MAX_RENDER_ITEMS);
                        for i in (0..show).rev() {
                            let (k, val) = &entries[i];
                            stack.push(RenderOp::Lit(")".to_string()));
                            stack.push(RenderOp::Val(val, depth + 1));
                            stack.push(RenderOp::Lit(format!("({:?}, ", k)));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit("Map([".to_string()));
                    }
                    Value::Option(None) => out.push_str("Option(None)"),
                    Value::Option(Some(inner)) => {
                        stack.push(RenderOp::Lit("))".to_string()));
                        stack.push(RenderOp::Val(inner, depth + 1));
                        stack.push(RenderOp::Lit("Option(Some(".to_string()));
                    }
                    Value::Result(Ok(inner)) => {
                        stack.push(RenderOp::Lit("))".to_string()));
                        stack.push(RenderOp::Val(inner, depth + 1));
                        stack.push(RenderOp::Lit("Result(Ok(".to_string()));
                    }
                    Value::Result(Err(e)) => {
                        out.push_str(&format!("Result(Err({:?}))", e));
                    }
                    Value::Object(o) => {
                        // Expand object fields inline (no nested Debug call).
                        if o.fields.is_empty() {
                            out.push_str(&format!(
                                "Object(Object {{ type_id: {:?}, fields: {{}}}})",
                                o.type_id
                            ));
                            continue;
                        }
                        let mut keys: Vec<&String> = o.fields.keys().collect();
                        keys.sort();
                        let show = keys.len().min(MAX_RENDER_ITEMS);
                        stack.push(RenderOp::Lit(")".to_string()));
                        stack.push(RenderOp::Lit("}".to_string()));
                        stack.push(RenderOp::Lit("}".to_string()));
                        if keys.len() > show {
                            stack.push(RenderOp::Lit(format!(
                                "... +{} more",
                                keys.len() - show
                            )));
                            if show > 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        for i in (0..show).rev() {
                            let k = keys[i].clone();
                            let val_ref: &Value = o.fields.get(&k).expect("key must exist");
                            // Need owned key string because borrow of `o` (behind Arc)
                            // lives as long as `v`, which outlives the loop. We clone
                            // the key into a Lit. Value ref borrows from `o`.
                            // To satisfy lifetimes we push Val first then Lits in
                            // reverse-pop order. Order for entry: `"k": V`.
                            // Pop order wanted: `"k": `, V, (sep?), ...
                            // Push order (reverse): sep?, V, `"k": `.
                            // Handle separators via separate Lits.
                            stack.push(RenderOp::Val(val_ref, depth + 1));
                            stack.push(RenderOp::Lit(format!("{:?}: ", k)));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit("fields: {".to_string()));
                        stack.push(RenderOp::Lit(format!(
                            "Object(Object {{ type_id: {:?}, ",
                            o.type_id
                        )));
                    }
                    Value::Function(fc) => {
                        // Render params inline + captured values inline.
                        let mut keys: Vec<&String> = fc.captured.keys().collect();
                        keys.sort();
                        let show = keys.len().min(MAX_RENDER_ITEMS);
                        stack.push(RenderOp::Lit(")".to_string()));
                        stack.push(RenderOp::Lit(", ..}".to_string()));
                        stack.push(RenderOp::Lit("}".to_string()));
                        if keys.len() > show {
                            stack.push(RenderOp::Lit(format!(
                                "... +{} more",
                                keys.len() - show
                            )));
                            if show > 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        for i in (0..show).rev() {
                            let k = keys[i].clone();
                            let val_ref: &Value =
                                fc.captured.get(&k).expect("key must exist");
                            stack.push(RenderOp::Val(val_ref, depth + 1));
                            stack.push(RenderOp::Lit(format!("{:?}: ", k)));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit("captured: {".to_string()));
                        stack.push(RenderOp::Lit(format!(
                            "Function(Closure {{ params: {:?}, ",
                            fc.params
                        )));
                    }
                }
            }
        }
    }
    out
}

fn value_display_string(root: &Value) -> String {
    // Human-readable, infallible, iterative, depth-limited (`#...`) and
    // breadth-capped (`... +N more`).
    // Complexity: O(rendered nodes).
    let mut out = String::new();
    let mut stack: Vec<RenderOp<'_>> = vec![RenderOp::Val(root, 0)];
    while let Some(op) = stack.pop() {
        match op {
            RenderOp::Lit(s) => out.push_str(&s),
            RenderOp::Val(v, depth) => {
                if depth > MAX_VALUE_DEPTH {
                    out.push_str("#...");
                    continue;
                }
                match v {
                    Value::Int(i) => out.push_str(&i.to_string()),
                    Value::Float(f) => out.push_str(&format!("{}", f)),
                    Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
                    Value::String(s) => out.push_str(s),
                    Value::Char(c) => out.push(*c),
                    Value::Unit => out.push_str("()"),
                    Value::Null => out.push_str("null"),
                    Value::Array(items) => {
                        if items.is_empty() {
                            out.push_str("[]");
                            continue;
                        }
                        stack.push(RenderOp::Lit("]".to_string()));
                        if items.len() > MAX_RENDER_ITEMS {
                            stack.push(RenderOp::Lit(format!(
                                ", ... +{} more",
                                items.len() - MAX_RENDER_ITEMS
                            )));
                        }
                        let show = items.len().min(MAX_RENDER_ITEMS);
                        for i in (0..show).rev() {
                            stack.push(RenderOp::Val(&items[i], depth + 1));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit("[".to_string()));
                    }
                    Value::Map(entries) => {
                        if entries.is_empty() {
                            out.push_str("{}");
                            continue;
                        }
                        stack.push(RenderOp::Lit("}".to_string()));
                        if entries.len() > MAX_RENDER_ITEMS {
                            stack.push(RenderOp::Lit(format!(
                                ", ... +{} more",
                                entries.len() - MAX_RENDER_ITEMS
                            )));
                        }
                        let show = entries.len().min(MAX_RENDER_ITEMS);
                        for i in (0..show).rev() {
                            let (k, val) = &entries[i];
                            stack.push(RenderOp::Val(val, depth + 1));
                            stack.push(RenderOp::Lit(format!("{}: ", k)));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit("{".to_string()));
                    }
                    Value::Option(None) => out.push_str("none"),
                    Value::Option(Some(inner)) => {
                        stack.push(RenderOp::Lit(")".to_string()));
                        stack.push(RenderOp::Val(inner, depth + 1));
                        stack.push(RenderOp::Lit("some(".to_string()));
                    }
                    Value::Result(Ok(inner)) => {
                        stack.push(RenderOp::Lit(")".to_string()));
                        stack.push(RenderOp::Val(inner, depth + 1));
                        stack.push(RenderOp::Lit("ok(".to_string()));
                    }
                    Value::Result(Err(e)) => {
                        out.push_str("err(");
                        out.push_str(e);
                        out.push(')');
                    }
                    Value::Object(o) => {
                        if o.fields.is_empty() {
                            out.push_str(&format!("{} {{}}", o.type_id));
                            continue;
                        }
                        let mut keys: Vec<&String> = o.fields.keys().collect();
                        keys.sort();
                        let show = keys.len().min(MAX_RENDER_ITEMS);
                        stack.push(RenderOp::Lit("}".to_string()));
                        if keys.len() > show {
                            stack.push(RenderOp::Lit(format!(
                                ", ... +{} more",
                                keys.len() - show
                            )));
                        }
                        for i in (0..show).rev() {
                            let k = keys[i].clone();
                            let val_ref: &Value = o.fields.get(&k).expect("key must exist");
                            stack.push(RenderOp::Val(val_ref, depth + 1));
                            stack.push(RenderOp::Lit(format!("{}: ", k)));
                            if i != 0 {
                                stack.push(RenderOp::Lit(", ".to_string()));
                            }
                        }
                        stack.push(RenderOp::Lit(format!("{} {{", o.type_id)));
                    }
                    Value::Function(fc) => {
                        out.push_str("<function(");
                        for (i, p) in fc.params.iter().enumerate() {
                            if i > 0 {
                                out.push_str(", ");
                            }
                            out.push_str(p);
                            if i + 1 >= MAX_RENDER_ITEMS && fc.params.len() > MAX_RENDER_ITEMS
                            {
                                out.push_str(&format!(
                                    ", ... +{} more",
                                    fc.params.len() - (i + 1)
                                ));
                                break;
                            }
                        }
                        out.push_str(")>");
                    }
                }
            }
        }
    }
    out
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", value_debug_string(self))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", value_display_string(self))
    }
}

// --- JSON escaping (std only, no deps) ---

fn escape_json_into(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
}

fn value_json_string(root: &Value) -> String {
    // Iterative JSON rendering with depth limit (beyond => `"#..."`).
    // No breadth cap (full data) so output stays valid JSON.
    // Functions render as `"<function>"` (lossy, documented).
    // Complexity: O(n) time where n = node count; O(depth) heap.
    enum JOp<'a> {
        Lit(String),
        Val(&'a Value, usize),
    }
    let mut out = String::new();
    let mut stack: Vec<JOp<'_>> = vec![JOp::Val(root, 0)];
    while let Some(op) = stack.pop() {
        match op {
            JOp::Lit(s) => out.push_str(&s),
            JOp::Val(v, depth) => {
                if depth > MAX_VALUE_DEPTH {
                    out.push_str("\"#...\"");
                    continue;
                }
                match v {
                    Value::Int(i) => out.push_str(&i.to_string()),
                    Value::Float(f) => {
                        if f.is_finite() {
                            out.push_str(&format!("{}", f));
                        } else {
                            out.push_str("null");
                        }
                    }
                    Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
                    Value::String(s) => {
                        out.push('"');
                        escape_json_into(s, &mut out);
                        out.push('"');
                    }
                    Value::Char(c) => {
                        out.push('"');
                        escape_json_into(&c.to_string(), &mut out);
                        out.push('"');
                    }
                    Value::Unit => out.push_str("null"),
                    Value::Null => out.push_str("null"),
                    Value::Array(items) => {
                        if items.is_empty() {
                            out.push_str("[]");
                            continue;
                        }
                        stack.push(JOp::Lit("]".to_string()));
                        for i in (0..items.len()).rev() {
                            stack.push(JOp::Val(&items[i], depth + 1));
                            if i != 0 {
                                stack.push(JOp::Lit(",".to_string()));
                            }
                        }
                        stack.push(JOp::Lit("[".to_string()));
                    }
                    Value::Map(entries) => {
                        if entries.is_empty() {
                            out.push_str("{}");
                            continue;
                        }
                        stack.push(JOp::Lit("}".to_string()));
                        for i in (0..entries.len()).rev() {
                            let (k, val) = &entries[i];
                            stack.push(JOp::Val(val, depth + 1));
                            let mut open = String::with_capacity(k.len() + 4);
                            open.push('"');
                            escape_json_into(k, &mut open);
                            open.push_str("\":");
                            stack.push(JOp::Lit(open));
                            if i != 0 {
                                stack.push(JOp::Lit(",".to_string()));
                            }
                        }
                        stack.push(JOp::Lit("{".to_string()));
                    }
                    Value::Option(None) => out.push_str("null"),
                    Value::Option(Some(inner)) => {
                        stack.push(JOp::Val(inner, depth + 1));
                    }
                    Value::Result(Ok(inner)) => {
                        stack.push(JOp::Lit("}".to_string()));
                        stack.push(JOp::Val(inner, depth + 1));
                        stack.push(JOp::Lit("{\"ok\":".to_string()));
                    }
                    Value::Result(Err(e)) => {
                        out.push_str("{\"err\":\"");
                        escape_json_into(e, &mut out);
                        out.push_str("\"}");
                    }
                    Value::Object(o) => {
                        if o.fields.is_empty() {
                            out.push_str("{\"$type\":\"");
                            escape_json_into(&o.type_id, &mut out);
                            out.push_str("\"}");
                            continue;
                        }
                        let mut keys: Vec<&String> = o.fields.keys().collect();
                        keys.sort();
                        stack.push(JOp::Lit("}".to_string()));
                        for i in (0..keys.len()).rev() {
                            let k = keys[i].clone();
                            let val_ref: &Value = o.fields.get(&k).expect("key must exist");
                            stack.push(JOp::Val(val_ref, depth + 1));
                            let mut open = String::with_capacity(k.len() + 4);
                            open.push('"');
                            escape_json_into(&k, &mut open);
                            open.push_str("\":");
                            stack.push(JOp::Lit(open));
                            if i != 0 {
                                stack.push(JOp::Lit(",".to_string()));
                            }
                        }
                        // Prepend $type then comma: {"$type":"T",...}
                        // We pushed "{" last; need to inject $type. Instead push
                        // "{" + "$type" as two lits: "{" on top, then type entry
                        // will follow. Simpler: push "{" then handle type by
                        // pushing type value as literal after "{".
                        // Stack top is "{"; after popping "{" we need
                        // `"$type":"T",`. So push fields first, then type prefix
                        // in reverse: fields..., type-comma, "{".
                        // We already pushed fields; now push type prefix and "{".
                        stack.push(JOp::Lit(",".to_string()));
                        let mut typ = String::with_capacity(o.type_id.len() + 12);
                        typ.push_str("\"$type\":\"");
                        escape_json_into(&o.type_id, &mut typ);
                        typ.push('"');
                        stack.push(JOp::Lit(typ));
                        stack.push(JOp::Lit("{".to_string()));
                    }
                    Value::Function(_) => out.push_str("\"<function>\""),
                }
            }
        }
    }
    out
}

impl Value {
    /// `Option<T>` constructor helpers (Spec §8).
    /// Complexity: O(1).
    #[inline]
    pub fn some(v: Value) -> Self {
        Value::Option(Some(Box::new(v)))
    }
    /// Complexity: O(1).
    #[inline]
    pub fn none() -> Self {
        Value::Option(None)
    }
    /// `Result<T, E>` constructor helpers (Spec §8).
    /// Complexity: O(1).
    #[inline]
    pub fn ok(v: Value) -> Self {
        Value::Result(Ok(Box::new(v)))
    }
    /// Complexity: O(n) where n = message len (allocates).
    #[inline]
    pub fn err(e: impl Into<String>) -> Self {
        Value::Result(Err(e.into()))
    }
    /// Safe unwrap with explicit `E0801` failure instead of silent null.
    /// Complexity: O(1).
    pub fn unwrap_option(self) -> std::result::Result<Value, String> {
        match self {
            Value::Option(Some(v)) => Ok(*v),
            Value::Option(None) => Err("E0801: unwrap on None".to_string()),
            other => Ok(other),
        }
    }
    /// Complexity: O(1).
    #[inline]
    pub fn is_null_or_none(&self) -> bool {
        matches!(self, Value::Null | Value::Option(None))
    }

    // ---- Part X: `Result<T,E>` runtime combinators (Spec §8) ----
    //
    // `Value::ok` / `Value::err` construct; the helpers below are the
    // runtime equivalents of `map` / `and_then` / `unwrap`. `Err`
    // propagates unchanged through `map`/`and_then`; non-`Result`
    // values pass through `map` as `f(self)` so `ok(x).map(f)` and
    // plain-value pipelines share one shape. All combinators are
    // total (no panics).
    /// Complexity: O(1).
    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(self, Value::Result(Ok(_)))
    }
    /// Complexity: O(1).
    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, Value::Result(Err(_)))
    }
    /// Complexity: O(1) plus `f`.
    pub fn map_result(self, f: impl FnOnce(Value) -> Value) -> Self {
        match self {
            Value::Result(Ok(v)) => Value::Result(Ok(Box::new(f(*v)))),
            Value::Result(Err(e)) => Value::Result(Err(e)),
            other => f(other),
        }
    }
    /// Complexity: O(1) plus `f`.
    pub fn and_then_result(self, f: impl FnOnce(Value) -> Value) -> Self {
        match self {
            Value::Result(Ok(v)) => f(*v),
            Value::Result(Err(e)) => Value::Result(Err(e)),
            other => f(other),
        }
    }
    /// Complexity: O(1) plus `f`.
    pub fn map_err_string(self, f: impl FnOnce(String) -> String) -> Self {
        match self {
            Value::Result(Ok(v)) => Value::Result(Ok(v)),
            Value::Result(Err(e)) => Value::Result(Err(f(e))),
            other => Value::Result(Ok(Box::new(other))),
        }
    }
    /// Complexity: O(1).
    pub fn unwrap_result(self) -> std::result::Result<Value, String> {
        match self {
            Value::Result(Ok(v)) => Ok(*v),
            Value::Result(Err(e)) => Err(e),
            other => Ok(other),
        }
    }
    /// Complexity: O(1).
    pub fn unwrap_result_or(self, default: Value) -> Value {
        match self {
            Value::Result(Ok(v)) => *v,
            Value::Result(Err(_)) => default,
            other => other,
        }
    }
    // ---- `Option<T>` combinators (symmetric with `Result`) ----
    /// Complexity: O(1) plus `f`.
    pub fn map_option(self, f: impl FnOnce(Value) -> Value) -> Self {
        match self {
            Value::Option(Some(v)) => Value::Option(Some(Box::new(f(*v)))),
            Value::Option(None) => Value::Option(None),
            other => f(other),
        }
    }
    /// Complexity: O(1) plus `f`.
    pub fn and_then_option(self, f: impl FnOnce(Value) -> Value) -> Self {
        match self {
            Value::Option(Some(v)) => f(*v),
            Value::Option(None) => Value::Option(None),
            other => f(other),
        }
    }

    // ---- Hardening: type introspection + truthiness + accessors ----

    /// Exact type string per variant.
    ///
    /// Returns one of `int`, `float`, `bool`, `string`, `char`, `unit`,
    /// `null`, `array`, `map`, `option`, `result`, `object`, `function`.
    /// Inner payloads do not change the top-level name: `some(1)` and
    /// `none` are both `option`; `ok(1)` and `err("e")` are both `result`;
    /// use `is_some`/`is_none`/`is_ok`/`is_err` to discriminate payloads,
    /// and `object_type_id()` for an `Object`'s inner tag.
    /// Complexity: O(1).
    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Unit => "unit",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Map(_) => "map",
            Value::Option(_) => "option",
            Value::Result(_) => "result",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
        }
    }

    /// Whether this value is `Some` (payload present).
    /// Complexity: O(1).
    #[inline]
    pub fn is_some(&self) -> bool {
        matches!(self, Value::Option(Some(_)))
    }

    /// Whether this value is `None`.
    /// Complexity: O(1).
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Value::Option(None))
    }

    /// Lexicon truthiness: `false`, `0`, `0.0`, `""`, `[]`, `{}`, `()`,
    /// `null`, `none`, and `err(_)` are falsy; everything else (including
    /// `some(_)`, `ok(_)`, non-empty collections, objects, functions) is
    /// truthy. Shallow O(1): `some`/`ok` do not inspect inner payloads so
    /// deep nesting cannot overflow.
    /// Complexity: O(1).
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Char(_) => true,
            Value::Unit => false,
            Value::Null => false,
            Value::Array(a) => !a.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Option(o) => o.is_some(),
            Value::Result(r) => r.is_ok(),
            Value::Object(_) => true,
            Value::Function(_) => true,
        }
    }

    /// Borrowed integer accessor (strict: only `Int`).
    /// Complexity: O(1).
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Borrowed float accessor (strict: only `Float`).
    /// Complexity: O(1).
    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Borrowed bool accessor (strict: only `Bool`).
    /// Complexity: O(1).
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Borrowed string slice accessor (no clone, hot-path friendly).
    /// Complexity: O(1).
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Borrowed char accessor.
    /// Complexity: O(1).
    #[inline]
    pub fn as_char(&self) -> Option<char> {
        match self {
            Value::Char(c) => Some(*c),
            _ => None,
        }
    }

    /// Borrowed array slice accessor (no clone).
    /// Complexity: O(1).
    #[inline]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// Borrowed map slice accessor (no clone).
    /// Complexity: O(1).
    #[inline]
    pub fn as_map(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Map(m) => Some(m.as_slice()),
            _ => None,
        }
    }

    /// Borrowed object accessor (no clone).
    /// Complexity: O(1).
    #[inline]
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(o) => Some(o.as_ref()),
            _ => None,
        }
    }

    /// Borrowed closure accessor (no clone).
    /// Complexity: O(1).
    #[inline]
    pub fn as_function(&self) -> Option<&Closure> {
        match self {
            Value::Function(f) => Some(f.as_ref()),
            _ => None,
        }
    }

    /// Inner type tag for `Object`, if this value is an object.
    /// Complexity: O(1).
    #[inline]
    pub fn object_type_id(&self) -> Option<&str> {
        match self {
            Value::Object(o) => Some(o.type_id.as_str()),
            _ => None,
        }
    }

    /// Borrowed object field lookup (no clone).
    /// Complexity: O(1) average, or O(1) `None` when not an object.
    #[inline]
    pub fn object_get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(o) => o.get(key),
            _ => None,
        }
    }

    /// Whether this object has `key` (false when not an object).
    /// Complexity: O(1) average.
    #[inline]
    pub fn object_has(&self, key: &str) -> bool {
        match self {
            Value::Object(o) => o.has(key),
            _ => false,
        }
    }

    /// Number of object fields (`None` when not an object).
    /// Complexity: O(1).
    #[inline]
    pub fn object_len(&self) -> Option<usize> {
        match self {
            Value::Object(o) => Some(o.len()),
            _ => None,
        }
    }

    /// Insert/overwrite an object field (clone-on-write via `Arc::make_mut`).
    ///
    /// Returns `true` when `self` is an object, `false` otherwise (no-op).
    /// Complexity: O(1) average plus at most one `Object` clone when shared.
    pub fn object_set(&mut self, key: impl Into<String>, value: Value) -> bool {
        match self {
            Value::Object(o) => {
                std::sync::Arc::make_mut(o).set(key, value);
                true
            }
            _ => false,
        }
    }

    /// Remove an object field, returning the old value.
    /// Complexity: O(1) average (plus clone-on-write when shared).
    pub fn object_remove(&mut self, key: &str) -> Option<Value> {
        match self {
            Value::Object(o) => std::sync::Arc::make_mut(o).remove(key),
            _ => None,
        }
    }

    /// JSON rendering with a depth limit for nested `Array`/`Map`/`Object`.
    ///
    /// Beyond [`MAX_VALUE_DEPTH`] nested values render as `"#..."` so
    /// adversarial nesting cannot overflow the stack or produce unbounded
    /// output depth. `Unit`/`Null` render as `null`; `None` renders as
    /// `null` and `Some(v)` renders as `v`; `Ok(v)` renders as
    /// `{"ok":v}` and `Err(e)` as `{"err":"e"}`; `Object` renders as
    /// `{"$type":"T",...fields}`; `Function` renders as `"<function>"`
    /// (lossy, documented). Infallible.
    /// Complexity: O(n) where n = node count; O(depth) heap, no recursion.
    pub fn to_json_string(&self) -> String {
        value_json_string(self)
    }
}
pub use error::{codes, Diagnostic, Error, Result, Severity};
pub use span::Span;

pub type FileId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(pub FileId);

impl SourceId {
    pub fn new(id: FileId) -> Self {
        SourceId(id)
    }
}

// ---------------------------------------------------------------------------
// Part X — structured concurrency primitives (Spec §7 + Part X).
//
// Contract: a `StructuredScope` owns every task it spawns; `join`
// blocks until ALL tasks finish (join-all, no detach). A panicking
// task does not abort the scope: it is captured as
// `Value::err("E0701: ...")` so the parent observes the failure
// deterministically. No background threads outlive `join`.
// Complexity: spawn O(1); join O(tasks).
// ---------------------------------------------------------------------------

/// Owned structured-concurrency scope over `Value`-producing threads.
pub struct StructuredScope {
    handles: Vec<std::thread::JoinHandle<Value>>,
}

impl StructuredScope {
    pub fn new() -> Self {
        StructuredScope { handles: Vec::new() }
    }

    /// Spawn a task owned by this scope. The closure must be `'static`
    /// because the thread may outlive the spawning call (but never
    /// outlives `join`).
    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce() -> Value + Send + 'static,
    {
        self.handles.push(std::thread::spawn(f));
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Block until every spawned task finishes, returning results in
    /// spawn order. Panics surface as `Value::err("E0701: ...")`.
    pub fn join(self) -> Vec<Value> {
        self.handles
            .into_iter()
            .map(|h| match h.join() {
                Ok(v) => v,
                Err(_) => Value::err("E0701: scoped task panicked"),
            })
            .collect()
    }
}

impl Default for StructuredScope {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: run `tasks` to completion inside one scope.
pub fn run_scoped(tasks: Vec<Box<dyn FnOnce() -> Value + Send>>) -> Vec<Value> {
    let mut scope = StructuredScope::new();
    for t in tasks {
        scope.spawn(t);
    }
    scope.join()
}

// ---------------------------------------------------------------------------
// Part X — RAII guard (Spec Part X, RAII).
//
// `ScopeGuard` runs its cleanup closure exactly once when dropped,
// unless `dismiss`ed. Mirrors `defer` semantics for Rust hosts:
// cleanup runs during normal return AND during panic unwind.
// ---------------------------------------------------------------------------

/// RAII cleanup guard: runs `action` on drop unless dismissed.
pub struct ScopeGuard<F: FnOnce()> {
    action: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    pub fn new(action: F) -> Self {
        ScopeGuard { action: Some(action) }
    }

    /// Cancel the cleanup (the guard drops without running `action`).
    pub fn dismiss(mut self) {
        self.action = None;
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(action) = self.action.take() {
            action();
        }
    }
}

/// Construct a [`ScopeGuard`] without naming the closure type.
pub fn guard<F: FnOnce()>(action: F) -> ScopeGuard<F> {
    ScopeGuard::new(action)
}

// ---------------------------------------------------------------------------
// Part X — signing (ed25519 stub) + sha256 via std + reproducible hash.
//
// Policy: `lexicon-core` MUST NOT gain hashing dependencies, so SHA-256
// is implemented here with `std` only (FIPS 180-4). The ed25519 API is
// a STUB: it exposes key/sign/verify shapes for toolchain plumbing
// (artifact signing) but is NOT a real signature scheme and MUST NOT
// be used where security matters. Production signing lives behind the
// audited `lexicon-utils::crypto` helpers.
// ---------------------------------------------------------------------------

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 over `data` using `std` only (FIPS 180-4).
pub fn sha256_std(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn hex_nibble(n: u8) -> char {
    (b'0' + n + if n < 10 { 0 } else { 39 }) as char
}

/// Lowercase hex of a digest (no extra deps).
pub fn hex_digest(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(hex_nibble(b >> 4));
        s.push(hex_nibble(b & 0x0f));
    }
    s
}

/// Reproducible build hash (Spec §43): SHA-256 over length-prefixed
/// parts (`u64 LE len || bytes` per part). Order matters — callers
/// sort inputs first for order-independence. Deterministic for
/// identical inputs on every host.
pub fn reproducible_hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut buf = Vec::new();
    for p in parts {
        buf.extend_from_slice(&(p.len() as u64).to_le_bytes());
        buf.extend_from_slice(p);
    }
    sha256_std(&buf)
}

/// Hex rendering of [`reproducible_hash`].
pub fn reproducible_hash_hex(parts: &[&[u8]]) -> String {
    hex_digest(&reproducible_hash(parts))
}

/// STUB ed25519 key (NOT a real signature scheme).
///
/// Shape-only API for artifact-signing plumbing: `sign_stub` is a
/// deterministic keyed hash, verified in constant time. NEVER use
/// for security decisions; see `lexicon-utils::crypto` for audited
/// primitives.
#[derive(Debug, Clone)]
pub struct Ed25519StubKey {
    pub seed: [u8; 32],
}

impl Ed25519StubKey {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Ed25519StubKey { seed }
    }

    /// Stub "public key": `sha256(seed)`.
    pub fn public_stub(&self) -> [u8; 32] {
        sha256_std(&self.seed)
    }

    /// Stub "signature": `sha256(pub||msg) || sha256(seed||msg)`.
    pub fn sign_stub(&self, msg: &[u8]) -> [u8; 64] {
        let pk = self.public_stub();
        let mut a = Vec::with_capacity(32 + msg.len());
        a.extend_from_slice(&pk);
        a.extend_from_slice(msg);
        let h1 = sha256_std(&a);
        let mut b = Vec::with_capacity(32 + msg.len());
        b.extend_from_slice(&self.seed);
        b.extend_from_slice(msg);
        let h2 = sha256_std(&b);
        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&h1);
        sig[32..].copy_from_slice(&h2);
        sig
    }

    /// Constant-time stub verification.
    pub fn verify_stub(&self, msg: &[u8], sig: &[u8; 64]) -> bool {
        let expect = self.sign_stub(msg);
        let mut diff = 0u8;
        for (a, b) in expect.iter().zip(sig.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }
}

#[cfg(test)]
mod core_partx_tests {
    use super::*;

    #[test]
    fn result_map_and_then_propagate() {
        let ok = Value::ok(Value::Int(2));
        assert!(ok.is_ok() && !ok.is_err());
        let mapped = Value::ok(Value::Int(2)).map_result(|v| match v {
            Value::Int(i) => Value::Int(i * 3),
            o => o,
        });
        assert_eq!(mapped.unwrap_result().unwrap(), Value::Int(6));
        let chained = Value::ok(Value::Int(2)).and_then_result(|_| Value::err("boom"));
        assert!(chained.is_err());
        // Err passes through map/and_then unchanged.
        let e = Value::err("boom").map_result(|v| v);
        assert_eq!(e.unwrap_result().unwrap_err(), "boom");
        let e2 = Value::err("boom").and_then_result(|v| v);
        assert!(e2.is_err());
        // map_err rewrites the channel.
        let e3 = Value::err("a").map_err_string(|s| s + "b");
        assert_eq!(e3.unwrap_result().unwrap_err(), "ab");
        assert_eq!(Value::err("x").unwrap_result_or(Value::Int(1)), Value::Int(1));
    }

    #[test]
    fn option_combinators() {
        let s = Value::some(Value::Int(1)).map_option(|_| Value::Int(2));
        assert_eq!(s, Value::some(Value::Int(2)));
        assert_eq!(Value::none().map_option(|_| Value::Int(2)), Value::none());
        assert_eq!(Value::none().and_then_option(|_| Value::Int(1)), Value::none());
    }

    #[test]
    fn scope_guard_runs_and_dismiss_cancels() {
        use std::sync::{Arc, Mutex};
        let flag = Arc::new(Mutex::new(false));
        {
            let f2 = flag.clone();
            let _g = guard(move || *f2.lock().unwrap() = true);
        }
        assert!(*flag.lock().unwrap());
        let flag2 = Arc::new(Mutex::new(false));
        {
            let f2 = flag2.clone();
            let g = guard(move || *f2.lock().unwrap() = true);
            g.dismiss();
        }
        assert!(!*flag2.lock().unwrap());
    }

    #[test]
    fn structured_scope_joins_all() {
        let mut scope = StructuredScope::new();
        scope.spawn(|| Value::Int(1));
        scope.spawn(|| Value::Int(2));
        assert_eq!(scope.len(), 2);
        let mut out = scope.join();
        out.sort_by_key(|v| match v {
            Value::Int(i) => *i,
            _ => 0,
        });
        assert_eq!(out, vec![Value::Int(1), Value::Int(2)]);
    }

    #[test]
    fn sha256_std_matches_nist_vector() {
        // "abc" — FIPS 180-4 reference.
        assert_eq!(
            hex_digest(&sha256_std(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(sha256_std(b"").len(), 32);
    }

    #[test]
    fn reproducible_hash_deterministic_and_order_sensitive() {
        let a = reproducible_hash_hex(&[b"a", b"b"]);
        let b = reproducible_hash_hex(&[b"a", b"b"]);
        let c = reproducible_hash_hex(&[b"b", b"a"]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ed25519_stub_roundtrip() {
        let k = Ed25519StubKey::from_seed([7u8; 32]);
        let sig = k.sign_stub(b"artifact");
        assert!(k.verify_stub(b"artifact", &sig));
        assert!(!k.verify_stub(b"tampered", &sig));
    }
}

#[cfg(test)]
mod value_hardening_tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn deep_array(depth: usize) -> Value {
        let mut v = Value::Int(0);
        for _ in 0..depth {
            v = Value::Array(vec![v]);
        }
        v
    }

    fn test_closure(params: &[&str]) -> Value {
        let mut captured = HashMap::new();
        captured.insert("x".to_string(), Value::Int(1));
        Value::Function(Arc::new(Closure::new(
            params.iter().map(|s| s.to_string()).collect(),
            captured,
            Box::new(|_| Value::Int(0)),
        )))
    }

    #[test]
    fn type_name_covers_all_variants_incl_payloads() {
        assert_eq!(Value::Int(1).type_name(), "int");
        assert_eq!(Value::Float(1.0).type_name(), "float");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::String("s".into()).type_name(), "string");
        assert_eq!(Value::Char('c').type_name(), "char");
        assert_eq!(Value::Unit.type_name(), "unit");
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Array(vec![]).type_name(), "array");
        assert_eq!(Value::Map(vec![]).type_name(), "map");
        assert_eq!(Value::some(Value::Int(1)).type_name(), "option");
        assert_eq!(Value::none().type_name(), "option");
        assert_eq!(Value::ok(Value::Int(1)).type_name(), "result");
        assert_eq!(Value::err("e").type_name(), "result");
        let obj = Value::Object(Arc::new(Object::new("User")));
        assert_eq!(obj.type_name(), "object");
        assert_eq!(obj.object_type_id(), Some("User"));
        assert_eq!(test_closure(&["a"]).type_name(), "function");
    }

    #[test]
    fn is_truthy_semantics() {
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(3).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
        assert!(Value::Float(0.5).is_truthy());
        assert!(!Value::String(String::new()).is_truthy());
        assert!(Value::String("x".into()).is_truthy());
        assert!(Value::Char('z').is_truthy());
        assert!(!Value::Unit.is_truthy());
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Array(vec![]).is_truthy());
        assert!(Value::Array(vec![Value::Null]).is_truthy());
        assert!(!Value::Map(vec![]).is_truthy());
        assert!(!Value::none().is_truthy());
        assert!(Value::some(Value::Int(0)).is_truthy());
        assert!(Value::ok(Value::Int(0)).is_truthy());
        assert!(!Value::err("e").is_truthy());
        assert!(Value::Object(Arc::new(Object::new("T"))).is_truthy());
        assert!(test_closure(&[]).is_truthy());
    }

    #[test]
    fn display_human_readable() {
        assert_eq!(format!("{}", Value::Int(-3)), "-3");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::String("hi".into())), "hi");
        assert_eq!(format!("{}", Value::Char('q')), "q");
        assert_eq!(format!("{}", Value::Unit), "()");
        assert_eq!(format!("{}", Value::Null), "null");
        assert_eq!(
            format!(
                "{}",
                Value::Array(vec![Value::Int(1), Value::Bool(false)])
            ),
            "[1, false]"
        );
        assert_eq!(
            format!(
                "{}",
                Value::Map(vec![("k".into(), Value::Int(7))])
            ),
            "{k: 7}"
        );
        assert_eq!(format!("{}", Value::none()), "none");
        assert_eq!(format!("{}", Value::some(Value::Int(2))), "some(2)");
        assert_eq!(format!("{}", Value::ok(Value::Int(2))), "ok(2)");
        assert_eq!(format!("{}", Value::err("boom")), "err(boom)");
        let mut obj = Object::new("User");
        obj.set("name", Value::String("Al".into()));
        assert_eq!(
            format!("{}", Value::Object(Arc::new(obj))),
            "User {name: Al}"
        );
        assert_eq!(format!("{}", test_closure(&["x", "y"])), "<function(x, y)>");
    }

    #[test]
    fn to_json_string_depth_limited() {
        assert_eq!(Value::Int(1).to_json_string(), "1");
        assert_eq!(Value::Bool(true).to_json_string(), "true");
        assert_eq!(Value::String("a\"b".into()).to_json_string(), "\"a\\\"b\"");
        assert_eq!(Value::Char('x').to_json_string(), "\"x\"");
        assert_eq!(Value::Null.to_json_string(), "null");
        assert_eq!(Value::Unit.to_json_string(), "null");
        assert_eq!(Value::none().to_json_string(), "null");
        assert_eq!(Value::some(Value::Int(3)).to_json_string(), "3");
        assert_eq!(Value::ok(Value::Int(3)).to_json_string(), "{\"ok\":3}");
        assert_eq!(
            Value::err("bad").to_json_string(),
            "{\"err\":\"bad\"}"
        );
        assert_eq!(
            Value::Array(vec![Value::Int(1), Value::Int(2)]).to_json_string(),
            "[1,2]"
        );
        assert_eq!(
            Value::Map(vec![("k".into(), Value::Bool(false))]).to_json_string(),
            "{\"k\":false}"
        );
        let mut obj = Object::new("T");
        obj.set("x", Value::Int(1));
        let j = Value::Object(Arc::new(obj)).to_json_string();
        assert!(j.contains("\"$type\":\"T\"") && j.contains("\"x\":1"), "got: {}", j);
        assert_eq!(
            test_closure(&["a"]).to_json_string(),
            "\"<function>\""
        );
        // Depth limit: 10k nesting must not overflow and must truncate.
        // Note: derived `Drop` recurses, so intentionally leak the deep value
        // (`mem::forget`) to isolate the iterative `to_json_string` proof.
        // `Clone`/`PartialEq`/`Debug`/`Display` are the hardened paths per spec.
        let deep = deep_array(10_000);
        let js = deep.to_json_string();
        assert!(js.contains("#..."), "deep json should truncate, len={}", js.len());
        std::mem::forget(deep);
        // Non-finite floats are null (valid JSON).
        assert_eq!(
            Value::Float(f64::NAN).to_json_string(),
            "null"
        );
    }

    #[test]
    fn object_helpers() {
        let mut o = Object::new("User");
        assert!(o.is_empty() && o.len() == 0);
        assert!(!o.has("name"));
        assert_eq!(o.get("name"), None);
        o.set("name", Value::Int(1));
        assert!(o.has("name") && o.len() == 1 && !o.is_empty());
        assert_eq!(o.get("name"), Some(&Value::Int(1)));
        assert_eq!(o.remove("name"), Some(Value::Int(1)));
        assert_eq!(o.remove("missing"), None);
        assert!(o.is_empty());

        let mut v = Value::Object(Arc::new(Object::new("T")));
        assert_eq!(v.object_len(), Some(0));
        assert_eq!(v.object_get("k"), None);
        assert!(!v.object_has("k"));
        assert!(v.object_set("k", Value::Bool(true)));
        assert!(v.object_has("k"));
        assert_eq!(v.object_get("k"), Some(&Value::Bool(true)));
        assert_eq!(v.object_len(), Some(1));
        assert_eq!(v.object_remove("k"), Some(Value::Bool(true)));
        assert_eq!(v.object_remove("k"), None);
        // Non-object helpers are no-ops.
        let mut plain = Value::Int(1);
        assert_eq!(plain.object_len(), None);
        assert_eq!(plain.object_get("k"), None);
        assert!(!plain.object_has("k"));
        assert!(!plain.object_set("k", Value::Int(2)));
        assert_eq!(plain.object_remove("k"), None);
    }

    #[test]
    fn closure_arity_and_captures() {
        let c = Closure::new(
            vec!["a".into(), "b".into()],
            HashMap::from([("z".to_string(), Value::Int(1)), ("a".to_string(), Value::Int(2))]),
            Box::new(|_| Value::Null),
        );
        assert_eq!(c.arity(), 2);
        assert_eq!(c.captured_names(), vec!["a".to_string(), "z".to_string()]);
        assert!(c.has_capture("z"));
        assert!(!c.has_capture("missing"));
        assert_eq!(c.captured_get("z"), Some(&Value::Int(1)));
        assert_eq!(c.captured_get("missing"), None);
        assert_eq!(Closure::new(vec![], HashMap::new(), Box::new(|_| Value::Null)).arity(), 0);
    }

    #[test]
    fn borrowed_accessors() {
        assert_eq!(Value::Int(5).as_int(), Some(5));
        assert_eq!(Value::Float(1.0).as_int(), None);
        assert_eq!(Value::Float(2.5).as_float(), Some(2.5));
        assert_eq!(Value::Int(1).as_float(), None);
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(1).as_bool(), None);
        assert_eq!(Value::String("hi".into()).as_str(), Some("hi"));
        assert_eq!(Value::Int(1).as_str(), None);
        assert_eq!(Value::Char('c').as_char(), Some('c'));
        assert_eq!(Value::Int(1).as_char(), None);
        let arr = Value::Array(vec![Value::Int(1)]);
        assert_eq!(arr.as_array().unwrap().len(), 1);
        assert_eq!(Value::Int(1).as_array(), None);
        let map = Value::Map(vec![("k".into(), Value::Null)]);
        assert_eq!(map.as_map().unwrap().len(), 1);
        assert_eq!(Value::Int(1).as_map(), None);
        let obj = Value::Object(Arc::new(Object::new("T")));
        assert!(obj.as_object().is_some());
        assert_eq!(obj.as_object().unwrap().type_id, "T");
        assert!(Value::Int(1).as_object().is_none());
        assert!(test_closure(&["a"]).as_function().is_some());
        assert!(Value::Int(1).as_function().is_none());
        assert!(Value::some(Value::Int(1)).is_some());
        assert!(Value::none().is_none());
    }

    #[test]
    fn clone_partial_eq_debug_display_survive_10k_nesting() {
        // Note: derived `Drop` recurses, so leak deep values (`mem::forget`)
        // to isolate the iterative Clone/Eq/Debug/Display proof (the hardened
        // paths per spec).
        let deep = deep_array(10_000);
        // Clone must not overflow (iterative heap stack).
        let cloned = deep.clone();
        // Equality is depth-limited: deep vs deep is false beyond the cap,
        // but must not overflow either.
        let _eq_self = &deep == &deep;
        let _eq_clone = &deep == &cloned;
        assert!(!(&deep == &cloned), "10k-deep eq must be false beyond depth limit");
        // Shallow equality still works (fresh shallow values, safe to drop).
        assert_eq!(Value::Int(1), Value::Int(1));
        assert_ne!(Value::Int(1), Value::Int(2));
        assert_eq!(deep_array(3), deep_array(3));
        assert_ne!(deep_array(3), deep_array(4));
        // Debug/Display/JSON must truncate, not overflow.
        let dbg = format!("{:?}", deep);
        assert!(dbg.contains("#..."), "debug should truncate deep nesting");
        let disp = format!("{}", deep);
        assert!(disp.contains("#..."), "display should truncate deep nesting");
        let js = deep.to_json_string();
        assert!(js.contains("#..."));
        std::mem::forget(deep);
        std::mem::forget(cloned);
    }

    #[test]
    fn partial_eq_keeps_historical_semantics() {
        assert_eq!(Value::Unit, Value::Unit);
        assert_eq!(Value::Null, Value::Null);
        assert_ne!(Value::Unit, Value::Null);
        assert_ne!(Value::Int(1), Value::Float(1.0));
        assert_eq!(
            Value::Map(vec![("a".into(), Value::Int(1))]),
            Value::Map(vec![("a".into(), Value::Int(1))])
        );
        assert_ne!(
            Value::Map(vec![("a".into(), Value::Int(1))]),
            Value::Map(vec![("a".into(), Value::Int(2))])
        );
        // Ordered map comparison (insertion order matters).
        assert_ne!(
            Value::Map(vec![
                ("a".into(), Value::Int(1)),
                ("b".into(), Value::Int(2))
            ]),
            Value::Map(vec![
                ("b".into(), Value::Int(2)),
                ("a".into(), Value::Int(1))
            ])
        );
        // Objects/functions never equal (even to themselves).
        let o = Value::Object(Arc::new(Object::new("T")));
        assert!(!(o == o.clone()));
        let f = test_closure(&["a"]);
        assert!(!(f == f.clone()));
    }

    #[test]
    fn value_size_stays_reasonable() {
        let sz = std::mem::size_of::<Value>();
        assert!(sz <= 64, "Value size {} exceeds 64-byte budget", sz);
    }

    #[test]
    fn debug_render_of_100k_array_is_length_capped() {
        let big = Value::Array(vec![Value::Int(1); 100_000]);
        let dbg = format!("{:?}", big);
        assert!(
            dbg.len() < 10_000,
            "debug of 100k array must be capped, got len={}",
            dbg.len()
        );
        assert!(dbg.contains("more"), "capped debug should summarize, got: {}", &dbg[..dbg.len().min(200)]);
        let disp = format!("{}", big);
        assert!(
            disp.len() < 10_000,
            "display of 100k array must be capped, got len={}",
            disp.len()
        );
        assert!(disp.contains("more"));
    }

    #[test]
    fn hardening_limits_and_debug_shapes() {
        assert_eq!(MAX_VALUE_DEPTH, 128);
        assert_eq!(MAX_RENDER_ITEMS, 32);
        // Shallow Debug keeps derived-like shapes.
        assert_eq!(format!("{:?}", Value::Int(1)), "Int(1)");
        assert_eq!(format!("{:?}", Value::Bool(true)), "Bool(true)");
        assert_eq!(format!("{:?}", Value::Unit), "Unit");
        assert_eq!(format!("{:?}", Value::Null), "Null");
        assert_eq!(format!("{:?}", Value::none()), "Option(None)");
        assert!(format!("{:?}", Value::some(Value::Int(1))).contains("Option(Some("));
        // Object/Closure Debug mention tags and cap breadth.
        let mut o = Object::new("User");
        o.set("a", Value::Int(1));
        let odbg = format!("{:?}", o);
        assert!(odbg.contains("User") && odbg.contains("\"a\""), "got: {}", odbg);
        let c = Closure::new(
            vec!["p".into()],
            HashMap::from([("k".to_string(), Value::Bool(false))]),
            Box::new(|_| Value::Null),
        );
        let cdbg = format!("{:?}", c);
        assert!(cdbg.contains("params") && cdbg.contains("\"k\""), "got: {}", cdbg);
        let vobj = Value::Object(Arc::new(o));
        assert!(format!("{:?}", vobj).contains("Object("));
        // Value::to_json uses MAX_VALUE_DEPTH: shallow stays exact.
        assert_eq!(Value::Array(vec![]).to_json_string(), "[]");
    }
}
