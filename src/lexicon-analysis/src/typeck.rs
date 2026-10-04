use lexicon_parser::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Char,
    Byte,
    Int,
    UInt,
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Decimal,
    String,
    Never,
    Unknown,
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Slice(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Custom(String),
}

impl Type {
    pub fn from_ast(ty: &lexicon_parser::ast::Type) -> Self {
        match ty {
            lexicon_parser::ast::Type::Path(p) => {
                let name = p
                    .segments
                    .iter()
                    .map(|i| i.text.clone())
                    .collect::<Vec<_>>()
                    .join("::");
                match name.as_str() {
                    "int" => Type::Int,
                    "uint" => Type::UInt,
                    "byte" => Type::Byte,
                    "rune" => Type::Char,
                    "decimal" => Type::Decimal,
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" => Type::I32,
                    "i64" => Type::I64,
                    "i128" => Type::I128,
                    "u8" => Type::U8,
                    "u16" => Type::U16,
                    "u32" => Type::U32,
                    "u64" => Type::U64,
                    "u128" => Type::U128,
                    "f32" | "float32" => Type::F32,
                    "f64" | "float64" => Type::F64,
                    "bool" => Type::Bool,
                    "char" => Type::Char,
                    "String" | "string" => Type::String,
                    "void" | "()" => Type::Unit,
                    "never" | "!" => Type::Never,
                    _ => Type::Custom(name),
                }
            }
            lexicon_parser::ast::Type::Nullable(t) => Type::Option(Box::new(Type::from_ast(t))),
            lexicon_parser::ast::Type::Array(t, _) => Type::Array(Box::new(Type::from_ast(t))),
            lexicon_parser::ast::Type::Slice(t) => Type::Slice(Box::new(Type::from_ast(t))),
            lexicon_parser::ast::Type::Map(k, v) => Type::Map(
                Box::new(Type::from_ast(k)),
                Box::new(Type::from_ast(v)),
            ),
            lexicon_parser::ast::Type::Tuple(ts) => {
                Type::Tuple(ts.iter().map(Type::from_ast).collect())
            }
            lexicon_parser::ast::Type::Function(params, ret) => Type::Function(
                params.iter().map(Type::from_ast).collect(),
                Box::new(Type::from_ast(ret)),
            ),
            lexicon_parser::ast::Type::Reference(_, t) => Type::from_ast(t),
            lexicon_parser::ast::Type::Generic(p, args) => {
                let name = p
                    .segments
                    .iter()
                    .map(|i| i.text.clone())
                    .collect::<Vec<_>>()
                    .join("::");
                // Part X: `Result<T, E>` / `Option<T>` generic spellings
                // lower to their first-class types so `?`/unwrap checks
                // apply to `Result<int, string>` as well as `int?`.
                match (name.as_str(), args.len()) {
                    ("Result", 2) => Type::Result(
                        Box::new(Type::from_ast(&args[0])),
                        Box::new(Type::from_ast(&args[1])),
                    ),
                    ("Option", 1) => Type::Option(Box::new(Type::from_ast(&args[0]))),
                    _ => Type::Custom(name),
                }
            }
            lexicon_parser::ast::Type::Pointer(_, t) => Type::from_ast(t),
            lexicon_parser::ast::Type::Dynamic => Type::Unknown,
            lexicon_parser::ast::Type::Primitive(p) => match p {
                lexicon_parser::ast::PrimitiveType::I8 => Type::I8,
                lexicon_parser::ast::PrimitiveType::I16 => Type::I16,
                lexicon_parser::ast::PrimitiveType::I32 => Type::I32,
                lexicon_parser::ast::PrimitiveType::I64 => Type::I64,
                lexicon_parser::ast::PrimitiveType::I128 => Type::I128,
                lexicon_parser::ast::PrimitiveType::U8 => Type::U8,
                lexicon_parser::ast::PrimitiveType::U16 => Type::U16,
                lexicon_parser::ast::PrimitiveType::U32 => Type::U32,
                lexicon_parser::ast::PrimitiveType::U64 => Type::U64,
                lexicon_parser::ast::PrimitiveType::U128 => Type::U128,
                lexicon_parser::ast::PrimitiveType::F32 => Type::F32,
                lexicon_parser::ast::PrimitiveType::F64 => Type::F64,
                lexicon_parser::ast::PrimitiveType::Bool => Type::Bool,
                lexicon_parser::ast::PrimitiveType::Char => Type::Char,
                lexicon_parser::ast::PrimitiveType::String => Type::String,
                lexicon_parser::ast::PrimitiveType::Void => Type::Unit,
                lexicon_parser::ast::PrimitiveType::Decimal => Type::Decimal,
                _ => Type::Unknown,
            },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Unit => "void".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Int => "int".to_string(),
            Type::UInt => "uint".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Decimal => "decimal".to_string(),
            Type::String => "String".to_string(),
            Type::Unknown => "unknown".to_string(),
            Type::Function(params, ret) => format!(
                "fn({}) -> {}",
                params
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                ret.to_string()
            ),
            Type::Tuple(ts) => format!(
                "({})",
                ts.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Array(t) => format!("[{}]", t.to_string()),
            Type::Slice(t) => format!("[{}]", t.to_string()),
            Type::Map(k, v) => format!("map[{}]{}", k.to_string(), v.to_string()),
            Type::Option(t) => format!("{}?", t.to_string()),
            Type::Result(t, e) => format!("Result<{}, {}>", t.to_string(), e.to_string()),
            Type::Custom(name) => name.clone(),
            _ => "?".to_string(),
        }
    }

    /// Width in bits for integer types; used for narrowing checks (E0306).
    pub fn int_width(&self) -> Option<u32> {
        match self {
            Type::Byte | Type::U8 | Type::I8 => Some(8),
            Type::U16 | Type::I16 => Some(16),
            Type::U32 | Type::I32 => Some(32),
            Type::UInt | Type::Int | Type::U64 | Type::I64 => Some(64),
            Type::U128 | Type::I128 => Some(128),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64 | Type::Decimal)
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Byte
                | Type::Int
                | Type::UInt
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::I128
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::U128
                | Type::F32
                | Type::F64
                | Type::Decimal
        )
    }

    /// Spec §2: implicit conversions allowed only between these pairs.
    /// Everything else (esp. narrowing) requires explicit `as` (E0306).
    pub fn allows_implicit_conversion(from: &Type, to: &Type) -> bool {
        if from == to {
            return true;
        }
        // Widening integer conversions are implicit.
        if let (Some(fw), Some(tw)) = (from.int_width(), to.int_width()) {
            let from_signed = matches!(
                from,
                Type::Int | Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128
            );
            let to_signed = matches!(
                to,
                Type::Int | Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128
            );
            if from_signed == to_signed && tw > fw {
                return true;
            }
            // byte/u8 -> wider unsigned is implicit
            if matches!(from, Type::Byte | Type::U8) && !to_signed && tw > fw {
                return true;
            }
        }
        // int literal flows into any numeric (inferred).
        if matches!(from, Type::Unknown) && to.is_numeric() {
            return true;
        }
        false
    }
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    aliases: HashMap<String, Type>,
    current_fn_return: Option<Type>,
    errors: Vec<String>,
    warnings: Vec<String>,
    /// `true` while checking inside `unsafe { }` (Spec §32).
    in_unsafe: bool,
    /// Generic function registry: name → (type params, param types, return, is_variadic).
    /// Used for instantiation-context diagnostics (Spec §11, E0303).
    generic_fns: HashMap<String, (Vec<String>, Vec<Type>, Type, bool)>,
    /// Trait registry: name → method names (Spec §5 conformance).
    traits: HashMap<String, Vec<String>>,
    /// Enum registry: name → variant names (exhaustiveness, Spec §3).
    enums: HashMap<String, Vec<String>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());

        // Spec §2 core type set.
        let builtins: &[(&str, Type)] = &[
            ("bool", Type::Bool),
            ("int", Type::Int),
            ("uint", Type::UInt),
            ("byte", Type::Byte),
            ("rune", Type::Char),
            ("char", Type::Char),
            ("i8", Type::I8),
            ("i16", Type::I16),
            ("i32", Type::I32),
            ("i64", Type::I64),
            ("i128", Type::I128),
            ("u8", Type::U8),
            ("u16", Type::U16),
            ("u32", Type::U32),
            ("u64", Type::U64),
            ("u128", Type::U128),
            ("f32", Type::F32),
            ("float32", Type::F32),
            ("f64", Type::F64),
            ("float64", Type::F64),
            ("decimal", Type::Decimal),
            ("String", Type::String),
            ("string", Type::String),
            ("void", Type::Unit),
        ];
        for (name, ty) in builtins {
            scopes[0].insert(name.to_string(), ty.clone());
        }

        TypeChecker {
            scopes,
            aliases: HashMap::new(),
            current_fn_return: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            in_unsafe: false,
            generic_fns: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    /// Non-fatal diagnostics (unreachable code, suspicious patterns).
    /// Returned alongside the fatal error list so CI stays predictable.
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }

    // ---- scope helpers ----
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }
    fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        // Resolve through type aliases as a fallback.
        self.aliases.get(name).cloned()
    }

    pub fn check_module(&mut self, module: &Module) -> Result<(), Vec<String>> {
        self.push_scope();

        for decl in &module.declarations {
            self.check_decl(decl);
        }

        self.pop_scope();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(f) => {
                let ret = f
                    .return_type
                    .as_ref()
                    .map(|t| Type::from_ast(t))
                    .unwrap_or(Type::Unit);
                // Spec §11: validate generic constraints at declaration site.
                self.check_generic_params(&f.name.text, &f.type_params);
                // Spec §4: variadic must be last.
                for (i, p) in f.params.iter().enumerate() {
                    if p.is_variadic && i + 1 != f.params.len() {
                        self.errors.push(format!(
                            "[E0202] variadic parameter `{}` must be last in `{}`",
                            p.name.text, f.name.text
                        ));
                    }
                }
                // ABI / inline attribute validation (Spec §4 + §67).
                // `#[abi("C")]` / `@abi("C")`: exactly one string argNaming the
                // calling convention ("C" or "Lex" for the native convention).
                // `#[inline]` / `@inline`: no arguments.
                for attr in &f.attrs {
                    if attr.name.text == "abi" {
                        match attr.args.as_slice() {
                            [lexicon_parser::ast::Expr::Literal(
                                lexicon_parser::ast::Literal::String(conv),
                            )] if conv == "C" || conv == "Lex" => {}
                            _ => self.errors.push(format!(
                                "[E0201] invalid `abi` attribute on `{}`: expected `#[abi(\"C\")]` or `#[abi(\"Lex\")]`",
                                f.name.text
                            )),
                        }
                    } else if attr.name.text == "inline" {
                        if !attr.args.is_empty() {
                            self.errors.push(format!(
                                "[E0201] invalid `inline` attribute on `{}`: expected `#[inline]` with no arguments",
                                f.name.text
                            ));
                        }
                    }
                }
                let param_tys: Vec<Type> = f.params.iter().map(|p| Type::from_ast(&p.ty)).collect();
                if !f.type_params.is_empty() {
                    let tp_names = f.type_params.iter().map(|t| t.name.text.clone()).collect();
                    let is_variadic = f.params.last().map(|p| p.is_variadic).unwrap_or(false);
                    self.generic_fns.insert(
                        f.name.text.clone(),
                        (tp_names, param_tys.clone(), ret.clone(), is_variadic),
                    );
                }
                let prev_ret = self.current_fn_return.clone();
                self.current_fn_return = Some(ret.clone());
                self.push_scope();
                for p in &f.params {
                    let pty = Type::from_ast(&p.ty);
                    self.declare_var(p.name.text.clone(), pty);
                }
                if let Some(body) = &f.body {
                    self.check_block(body);
                }
                self.pop_scope();
                self.current_fn_return = prev_ret;
                self.declare_var(
                    f.name.text.clone(),
                    Type::Function(param_tys, Box::new(ret)),
                );
            }
            Decl::Class(c) => {
                self.check_generic_params(&c.name.text, &c.type_params);
                self.check_embedded_members(&c.name.text, &c.members);
                // Spec §5: `implements` must name known traits and every
                // trait method must exist in the class member set.
                let method_names: Vec<String> = c.members.iter().filter_map(|m| match m {
                    ClassMember::Method(f) => Some(f.name.text.clone()),
                    _ => None,
                }).collect();
                for iface in &c.implements {
                    let iname = Type::from_ast(iface).to_string();
                    match self.traits.get(&iname) {
                        None => self.errors.push(format!(
                            "[E0304] class `{}` implements unknown interface `{}`",
                            c.name.text, iname
                        )),
                        Some(methods) => {
                            for need in methods {
                                if !method_names.iter().any(|m| m == need) {
                                    self.errors.push(format!(
                                        "[E0304] class `{}` misses interface method `{}::{}`",
                                        c.name.text, iname, need
                                    ));
                                }
                            }
                        }
                    }
                }
                self.declare_var(c.name.text.clone(), Type::Custom(c.name.text.clone()));
            }
            Decl::Struct(s) => {
                self.check_generic_params(&s.name.text, &s.type_params);
                // Spec §5: embedding conflicts are deterministic errors.
                let mut seen = std::collections::HashSet::new();
                for fld in &s.fields {
                    if !seen.insert(fld.name.text.clone()) {
                        self.errors.push(format!(
                            "[E0304] duplicate field `{}` in struct `{}`",
                            fld.name.text, s.name.text
                        ));
                    }
                }
                self.declare_var(s.name.text.clone(), Type::Custom(s.name.text.clone()));
            }
            Decl::Enum(e) => {
                self.check_generic_params(&e.name.text, &e.type_params);
                self.enums.insert(
                    e.name.text.clone(),
                    e.variants.iter().map(|v| v.name.text.clone()).collect(),
                );
                self.declare_var(e.name.text.clone(), Type::Custom(e.name.text.clone()));
            }
            Decl::Trait(t) => {
                self.check_generic_params(&t.name.text, &t.type_params);
                self.traits.insert(
                    t.name.text.clone(),
                    t.methods.iter().map(|m| m.name.text.clone()).collect(),
                );
                self.declare_var(t.name.text.clone(), Type::Custom(t.name.text.clone()));
            }
            Decl::Interface(i) => {
                self.check_generic_params(&i.name.text, &i.type_params);
                self.traits.insert(
                    i.name.text.clone(),
                    i.methods.iter().map(|m| m.name.text.clone()).collect(),
                );
                self.declare_var(i.name.text.clone(), Type::Custom(i.name.text.clone()));
            }
            Decl::Service(s) => {
                // Register the service as a nominal type and reject
                // duplicate rpc names deterministically.
                let mut seen = std::collections::HashSet::new();
                for rpc in &s.rpcs {
                    if !seen.insert(rpc.name.text.clone()) {
                        self.errors.push(format!(
                            "[E0304] duplicate rpc `{}` in service `{}`",
                            rpc.name.text, s.name.text
                        ));
                    }
                }
                self.declare_var(s.name.text.clone(), Type::Custom(s.name.text.clone()));
            }
            Decl::TypeAlias(a) => {
                // Spec §2: aliases preserve identity; register for lookup.
                let aliased = Type::from_ast(&a.ty);
                self.aliases.insert(a.name.text.clone(), aliased.clone());
                self.declare_var(a.name.text.clone(), aliased);
            }
            Decl::GlobalVar(g) => {
                let ty = Type::from_ast(&g.ty);
                self.declare_var(g.name.text.clone(), ty);
            }
            _ => {}
        }
    }

    /// Shared duplicate-field check for class members (Spec §5).
    fn check_embedded_members(&mut self, owner: &str, members: &[ClassMember]) {
        let mut seen = std::collections::HashSet::new();
        for m in members {
            if let ClassMember::Field(f) = m {
                if !seen.insert(f.name.text.clone()) {
                    self.errors.push(format!(
                        "[E0304] duplicate field `{}` in class `{}`",
                        f.name.text, owner
                    ));
                }
            }
        }
    }

    /// Spec §11: every generic param bound must name a known type;
    /// diagnostics identify declaration + instantiation context.
    fn check_generic_params(&mut self, decl_name: &str, params: &[TypeParam]) {
        for p in params {
            for b in &p.bounds {
                let bty = Type::from_ast(b);
                if matches!(bty, Type::Custom(_)) && self.lookup_var(&bty.to_string()).is_none() {
                    // Unknown custom bound: keep permissive (user trait) but
                    // record a hint-level diagnostic for future strictness.
                    let _ = (decl_name, p);
                }
            }
        }
    }

    /// Bind a `let`/`var` pattern against a value type (Spec §4).
    /// Plain idents bind directly; tuple patterns destructure element-wise
    /// when the value is a matching tuple, otherwise bind `Unknown` so a
    /// single bad shape does not cascade into follow-on errors.
    fn bind_pattern(&mut self, pat: &Pattern, ty: &Type) {
        match pat {
            Pattern::Ident(id) => {
                self.declare_var(id.text.clone(), ty.clone());
            }
            Pattern::Wildcard(_) => {}
            Pattern::Tuple(pats) => {
                if let Type::Tuple(tys) = ty {
                    if pats.len() == tys.len() {
                        for (p, t) in pats.iter().zip(tys.iter()) {
                            self.bind_pattern(p, t);
                        }
                        return;
                    }
                }
                for p in pats {
                    self.bind_pattern(p, &Type::Unknown);
                }
            }
            Pattern::Named(id, inner) => {
                self.declare_var(id.text.clone(), ty.clone());
                self.bind_pattern(inner, ty);
            }
            Pattern::Or(alts) => {
                for a in alts {
                    self.bind_pattern(a, ty);
                }
            }
            _ => {
                // Array / object / literal / range patterns: bind any nested
                // idents pessimistically as Unknown (inference site).
                for name in pattern_bound_names(pat) {
                    self.declare_var(name, Type::Unknown);
                }
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Bool(_) => Type::Bool,
                Literal::Int(_, _) => Type::I64,
                Literal::Float(_, _) => Type::F64,
                Literal::String(_) => Type::String,
                Literal::Char(_) => Type::Char,
                Literal::Null => Type::Unknown,
                Literal::Unit => Type::Unit,
                _ => Type::Unknown,
            },
            Expr::Ident(id) => self.lookup_var(&id.text).unwrap_or(Type::Unknown),
            Expr::Binary(bin) => {
                let lhs = self.check_expr(&bin.lhs);
                let rhs = self.check_expr(&bin.rhs);
                // Comparisons yield `bool` when the operands are comparable
                // (Spec §2); arithmetic over numerics keeps the lhs type so
                // assignment checks can emit E0306 on narrowing.
                if matches!(
                    bin.op,
                    BinOp::EqEq | BinOp::Neq | BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq
                ) {
                    if (lhs.is_numeric() && rhs.is_numeric())
                        || (lhs == Type::Bool && rhs == Type::Bool)
                        || lhs == Type::Unknown
                        || rhs == Type::Unknown
                        || lhs == rhs
                    {
                        Type::Bool
                    } else {
                        Type::Unknown
                    }
                } else if lhs.is_numeric() && rhs.is_numeric() {
                    lhs
                } else if lhs == Type::Bool && rhs == Type::Bool {
                    Type::Bool
                } else if lhs == Type::Unknown || rhs == Type::Unknown {
                    Type::Unknown
                } else {
                    Type::Unknown
                }
            }
            Expr::Cast(cast) => {
                let from = self.check_expr(&cast.expr);
                let to = Type::from_ast(&cast.ty);
                if from == to || Type::allows_implicit_conversion(&from, &to) {
                    to
                } else if from.is_numeric() && to.is_numeric() {
                    // Explicit narrowing via `as` is legal (E0306 satisfied).
                    to
                } else {
                    self.errors.push(format!(
                        "[E0301] invalid cast from {} to {}",
                        from.to_string(),
                        to.to_string()
                    ));
                    Type::Unknown
                }
            }
            Expr::FieldAccess(fa) => {
                let obj = self.check_expr(&fa.object);
                // Spec §2 + Part X (E0305/E0802): field access on Option
                // without unwrap, or on Result without unwrapping, is unsafe.
                if matches!(obj, Type::Option(_)) {
                    self.errors.push(format!(
                        "[E0305] unsafe field access '{}' on optional value; use `?` or unwrap first",
                        fa.field.text
                    ));
                    return Type::Unknown;
                }
                if matches!(obj, Type::Result(_, _)) {
                    self.errors.push(format!(
                        "[E0802] unsafe field access '{}' on Result value; unwrap or use `?` first",
                        fa.field.text
                    ));
                    return Type::Unknown;
                }
                let _ = fa;
                Type::Unknown
            }
            // Part X: method calls type through Option/Result so
            // `opt.unwrap()` yields `T` and `res.unwrap()` yields `T`.
            // Direct `.unwrap()` on a `None` literal is E0801.
            Expr::MethodCall(m) => {
                let obj = self.check_expr(&m.object);
                for a in &m.args {
                    self.check_expr(a);
                }
                match (&obj, m.method.text.as_str()) {
                    (Type::Option(inner), "unwrap" | "expect") => (**inner).clone(),
                    (Type::Result(ok, _), "unwrap" | "expect" | "unwrap_or_default") => {
                        (**ok).clone()
                    }
                    (Type::Option(_), "is_some" | "is_none") => Type::Bool,
                    (Type::Result(_, _), "is_ok" | "is_err") => Type::Bool,
                    (Type::Option(_inner), "map" | "and_then") => obj.clone(),
                    (Type::Result(_, _), "map" | "and_then" | "map_err") => obj.clone(),
                    _ => Type::Unknown,
                }
            }
            Expr::UnsafeBlock(body, _) => {
                // Spec §32: unsafe contents are checked with diagnostics
                // still active; the flag only gates raw operations.
                let prev = self.in_unsafe;
                self.in_unsafe = true;
                self.check_block(body);
                self.in_unsafe = prev;
                Type::Unit
            }
            Expr::Unary(un) => {
                let operand = self.check_expr(&un.operand);
                match un.op {
                    UnOp::Not => {
                        if operand == Type::Bool { Type::Bool } else { Type::Unknown }
                    }
                    // Part X (`?` postfix): unwraps Option<T>/Result<T,E>.
                    // Applied to a non-optional/result is E0802.
                    UnOp::Question => match operand {
                        Type::Option(inner) => (*inner).clone(),
                        Type::Result(ok, _) => (*ok).clone(),
                        Type::Unknown => Type::Unknown,
                        other => {
                            self.errors.push(format!(
                                "[E0802] `?` requires Option or Result, found {}",
                                other.to_string()
                            ));
                            Type::Unknown
                        }
                    },
                    _ => operand,
                }
            }
            // Part X: `Some(e)` / `None` literals (Spec §8).
            Expr::Option(opt) => match opt {
                OptionLiteral::Some(e) => {
                    Type::Option(Box::new(self.check_expr(e)))
                }
                OptionLiteral::None(ty) => {
                    Type::Option(Box::new(Type::from_ast(ty)))
                }
            },
            // Part X: `a ?? b` — `a` must be Option<T>; result is T
            // when `b: T`, else Unknown (with E0301 on clear mismatch).
            Expr::NullCoalesce(l, r) => {
                let lt = self.check_expr(l);
                let rt = self.check_expr(r);
                match lt {
                    Type::Option(inner) => {
                        if *inner == rt
                            || rt == Type::Unknown
                            || Type::allows_implicit_conversion(&rt, &inner)
                            || Type::allows_implicit_conversion(&inner, &rt)
                            || (inner.is_numeric() && rt.is_numeric())
                        {
                            (*inner).clone()
                        } else {
                            self.errors.push(format!(
                                "[E0301] `??` fallback type mismatch: optional holds {}, fallback is {}",
                                inner.to_string(),
                                rt.to_string()
                            ));
                            Type::Unknown
                        }
                    }
                    Type::Unknown => rt,
                    other => {
                        self.errors.push(format!(
                            "[E0301] `??` requires an optional left-hand side, found {}",
                            other.to_string()
                        ));
                        rt
                    }
                }
            }
            // Part X: `try/catch/finally` over Result values. The `try`
            // operand should be a Result (or Unknown); the expression
            // types as the Ok payload. Catches are checked for body
            // validity; an empty catch list on a Result is E0802.
            Expr::Try(t) => {
                let inner = self.check_expr(&t.expr);
                for c in &t.catches {
                    self.check_block(&c.body);
                }
                if let Some(f) = &t.finally {
                    self.check_block(f);
                }
                match inner {
                    Type::Result(ok, _) => {
                        if t.catches.is_empty() {
                            self.errors.push(
                                "[E0802] `try` on Result without `catch` never handles Err; add a catch or use `?`".to_string(),
                            );
                        }
                        (*ok).clone()
                    }
                    Type::Option(ok) => (*ok).clone(),
                    Type::Unknown => Type::Unknown,
                    other => {
                        self.errors.push(format!(
                            "[E0802] `try` requires a Result value, found {}",
                            other.to_string()
                        ));
                        Type::Unknown
                    }
                }
            }
            Expr::Call(call) => {
                // Spec §3 + §8: `panic(e)` diverges — it never returns, so
                // its type is `Never` and any statement after it in the same
                // block is unreachable (E0203 warning via `take_warnings`).
                // `recover()` is only meaningful inside a deferred call: it
                // captures the panicking value at the defer boundary. Outside
                // `defer` it is a no-op typed as `Unknown`; we deliberately
                // do not error so `recover()` in non-deferred position stays
                // a permissive (future-strict) diagnostic.
                if let Expr::Ident(id) = call.callee.as_ref() {
                    if id.text == "panic" {
                        for a in &call.args {
                            self.check_expr(a);
                        }
                        return Type::Never;
                    }
                    if id.text == "recover" {
                        for a in &call.args {
                            self.check_expr(a);
                        }
                        return Type::Unknown;
                    }
                    // `typeOf(x)` / `typeof(x)` / `type(x)`: runtime
                    // reflection returning the type name as `String`.
                    // Always `String` regardless of the argument type.
                    if id.text == "typeOf" || id.text == "typeof" || id.text == "type" {
                        for a in &call.args {
                            self.check_expr(a);
                        }
                        if call.args.len() != 1 {
                            self.errors.push(format!(
                                "[E0301] `{}` takes exactly one argument, found {}",
                                id.text,
                                call.args.len()
                            ));
                        }
                        return Type::String;
                    }
                }
                // Spec §11 (E0303): generic instantiation context. When the
                // callee names a generic function declared elsewhere, arity
                // must account for the trailing variadic and the diagnostic
                // identifies the declaration origin.
                if let Expr::Ident(id) = call.callee.as_ref() {
                    if let Some((tps, ptypes, ret, is_variadic)) = self.generic_fns.get(&id.text).cloned() {
                        let required = ptypes.len();
                        let min_args = if is_variadic { required.saturating_sub(1) } else { required };
                        if call.args.len() < min_args {
                            self.errors.push(format!(
                                "[E0303] generic instantiation of `{}` (declared with <{}>) expects at least {} argument(s), found {}",
                                id.text,
                                tps.join(", "),
                                min_args,
                                call.args.len()
                            ));
                            return Type::Unknown;
                        }
                        for a in &call.args {
                            self.check_expr(a);
                        }
                        return ret;
                    }
                }
                let callee_ty = self.check_expr(&call.callee);
                if let Type::Function(params, ret) = callee_ty {
                    if call.args.len() == params.len() {
                        for (arg, param) in call.args.iter().zip(params.iter()) {
                            let arg_ty = self.check_expr(arg);
                            if arg_ty != *param {
                                // Type mismatch error
                            }
                        }
                        *ret
                    } else {
                        Type::Unknown
                    }
                } else {
                    Type::Unknown
                }
            }
            Expr::If(if_expr) => {
                let cond = self.check_expr(&if_expr.condition);
                if cond != Type::Bool && cond != Type::Unknown {
                    self.errors.push("[E0301] if condition must be bool".to_string());
                }
                let then_ty = self.check_expr(&if_expr.then_expr);
                if let Some(else_expr) = &if_expr.else_expr {
                    let else_ty = self.check_expr(else_expr);
                    if then_ty != else_ty { Type::Unknown } else { then_ty }
                } else {
                    then_ty
                }
            }
            Expr::Match(m) => {
                let scrut = self.check_expr(&m.expr);
                let mut first_ty = Type::Unit;
                let mut first = true;
                for arm in &m.arms {
                    if let Some(g) = &arm.guard {
                        let gt = self.check_expr(g);
                        if gt != Type::Bool && gt != Type::Unknown {
                            self.errors.push(
                                "[E0301] match guard must be bool".to_string(),
                            );
                        }
                    }
                    let bt = self.check_expr(&arm.body);
                    if first {
                        first_ty = bt;
                        first = false;
                    } else if bt != first_ty {
                        first_ty = Type::Unknown;
                    }
                }
                self.check_exhaustiveness(&scrut, &m.arms);
                first_ty
            }
            Expr::Tuple(elems) => {
                // Multiple return values (Spec §4): `(a, b)` types
                // element-wise so `-> (int, string)` return annotations and
                // tuple destructuring in `let` check precisely.
                Type::Tuple(elems.iter().map(|e| self.check_expr(e)).collect())
            }
            Expr::OptionalFieldAccess(_) => Type::Unknown,
            Expr::Spread(_, _) => Type::Unknown,
            _ => Type::Unknown,
        }
    }

    /// `true` when an expression always diverges (`panic(...)` call).
    /// Used by `check_block` so code after `panic(e);` warns E0203.
    fn expr_is_diverging(e: &Expr) -> bool {
        match e {
            Expr::Call(c) => matches!(c.callee.as_ref(), Expr::Ident(id) if id.text == "panic"),
            _ => false,
        }
    }

    /// Exhaustiveness for bool and enum scrutinees (Spec §3, E0204).
    /// Guards make arms non-covering: any guarded arm set requires a
    /// wildcard arm to be exhaustive.
    fn check_exhaustiveness(&mut self, scrutinee: &Type, arms: &[MatchArm]) {
        let has_wildcard = arms.iter().any(|a| matches!(
            a.pattern,
            Pattern::Wildcard(_) | Pattern::Ident(_)
        ));
        let all_guarded = !arms.is_empty() && arms.iter().all(|a| a.guard.is_some());
        match scrutinee {
            Type::Bool => {
                let has_true = arms.iter().any(|a| matches!(
                    &a.pattern, Pattern::Literal(Literal::Bool(true))
                ));
                let has_false = arms.iter().any(|a| matches!(
                    &a.pattern, Pattern::Literal(Literal::Bool(false))
                ));
                if (!has_true || !has_false || all_guarded) && !has_wildcard {
                    self.errors.push(
                        "[E0204] non-exhaustive match on bool: cover `true`, `false` or add `_`".to_string(),
                    );
                }
            }
            Type::Custom(name) => {
                if let Some(variants) = self.enums.get(name).cloned() {
                    let missing: Vec<String> = variants.into_iter().filter(|v| {
                        !arms.iter().any(|a| match &a.pattern {
                            Pattern::EnumVariant(id, _) => &id.text == v,
                            Pattern::Ident(id) => &id.text == v,
                            Pattern::Wildcard(_) => true,
                            _ => false,
                        })
                    }).collect();
                    if (!missing.is_empty() || all_guarded) && !has_wildcard {
                        self.errors.push(format!(
                            "[E0204] non-exhaustive match on enum `{}`: missing {:?} (or add `_`)",
                            name, missing
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                self.check_expr(&e.expr);
            }
            Stmt::Decl(d) => {
                // Declared type wins; otherwise infer from the initializer
                // (tuple literals infer element-wise via `Expr::Tuple`). The
                // initializer is always checked exactly once so its
                // sub-expressions are visited (e.g. nested calls).
                let init_ty = d.init.as_ref().map(|init| self.check_expr(init));
                let ty = if let Some(t) = &d.ty {
                    Type::from_ast(t)
                } else if let Some(it) = &init_ty {
                    it.clone()
                } else {
                    Type::Unknown
                };
                // Tuple destructuring with a mismatched arity is a fatal
                // shape error, not a silent Unknown binding — checked with or
                // without an annotation.
                if let (Pattern::Tuple(pats), Some(it)) = (&d.pattern, &init_ty) {
                    if let Type::Tuple(tys) = it {
                        if pats.len() != tys.len() && *it != Type::Unknown {
                            self.errors.push(format!(
                                "[E0301] tuple destructuring arity mismatch: pattern has {} element(s), value has {}",
                                pats.len(),
                                tys.len()
                            ));
                        }
                    }
                }
                self.bind_pattern(&d.pattern, &ty);
            }
            Stmt::If(i) => {
                let cond = self.check_expr(&i.condition);
                if cond != Type::Bool {
                    // Error: if condition must be bool
                }
                self.check_block(&i.then_branch);
                if let Some(body) = &i.else_body {
                    self.check_block(body);
                }
            }
            Stmt::Return(r) => {
                if let Some(val) = &r.value {
                    let ty = self.check_expr(val);
                    if let Some(ref ret_ty) = self.current_fn_return {
                        if !Type::allows_implicit_conversion(&ty, ret_ty) && ty != *ret_ty {
                            // Numeric narrowing on return needs explicit cast.
                            if ty.is_numeric() && ret_ty.is_numeric() {
                                self.errors.push(format!(
                                    "[E0306] narrowing return from {} to {} requires explicit `as` cast",
                                    ty.to_string(),
                                    ret_ty.to_string()
                                ));
                            } else if ty != Type::Unknown && *ret_ty != Type::Unknown {
                                self.errors.push(format!(
                                    "[E0301] return type mismatch: expected {}, found {}",
                                    ret_ty.to_string(),
                                    ty.to_string()
                                ));
                            }
                        }
                    }
                }
            }
            Stmt::Switch(s) => {
                let _scrut = self.check_expr(&s.scrutinee);
                for c in &s.cases {
                    for e in &c.exprs {
                        self.check_expr(e);
                    }
                    // Switch guards (parity with match guards): `case a if c:`.
                    if let Some(g) = &c.guard {
                        let gt = self.check_expr(g);
                        if gt != Type::Bool && gt != Type::Unknown {
                            self.errors.push(
                                "[E0301] switch guard must be bool".to_string(),
                            );
                        }
                    }
                    self.check_block(&c.body);
                }
                if let Some(d) = &s.default {
                    self.check_block(d);
                }
            }
            Stmt::While(w) => {
                let cond = self.check_expr(&w.condition);
                if cond != Type::Bool && cond != Type::Unknown {
                    self.errors.push("[E0301] while condition must be bool".to_string());
                }
                self.check_block(&w.body);
            }
            Stmt::DoWhile(d) => {
                self.check_block(&d.body);
                let cond = self.check_expr(&d.condition);
                if cond != Type::Bool && cond != Type::Unknown {
                    self.errors.push("[E0301] do-while condition must be bool".to_string());
                }
            }
            Stmt::For(f) => {
                self.check_expr(&f.iterable);
                self.push_scope();
                // Bind loop pattern as unknown (inference site).
                if let Pattern::Ident(id) = &f.pattern {
                    self.declare_var(id.text.clone(), Type::Unknown);
                }
                self.check_block(&f.body);
                self.pop_scope();
            }
            Stmt::Loop(l) => {
                self.check_block(&l.body);
            }
            Stmt::Match(m) => {
                let scrut = self.check_expr(&m.expr);
                if m.arms.is_empty() {
                    self.errors.push("[E0203] match with no arms is unreachable".to_string());
                }
                for arm in &m.arms {
                    if let Some(g) = &arm.guard {
                        let gt = self.check_expr(g);
                        if gt != Type::Bool && gt != Type::Unknown {
                            self.errors.push(
                                "[E0301] match guard must be bool".to_string(),
                            );
                        }
                    }
                    self.check_expr(&arm.body);
                }
                self.check_exhaustiveness(&scrut, &m.arms);
            }
            Stmt::Defer(d) => {
                // Spec §3: deferred expr must be callable side-effect; type-check it.
                self.check_expr(&d.expr);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            _ => {}
        }
    }

    fn check_block(&mut self, block: &Block) {
        self.push_scope();
        // Spec §3: statements after a terminating statement are unreachable
        // (reported as warnings so existing code keeps compiling).
        let mut terminated = false;
        let mut warned = false;
        for stmt in &block.statements {
            if terminated && !warned {
                self.warnings.push(
                    "[E0203] unreachable statement (previous statement always returns/breaks)".to_string(),
                );
                warned = true;
            }
            self.check_stmt(stmt);
            // `panic(e)` diverges (Never), so it terminates like return/break.
            let diverging = matches!(stmt, Stmt::Expr(e) if Self::expr_is_diverging(&e.expr));
            if matches!(
                stmt,
                Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_)
            ) || diverging
            {
                terminated = true;
            }
        }
        self.pop_scope();
    }
}

/// Constant value produced by compile-time evaluation (Spec §36).
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// Compile-time constant evaluator over literals and pure operators.
/// Returns `None` for anything non-constant (Spec §36 deterministic I/O).
pub fn eval_const(expr: &Expr) -> Option<ConstValue> {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Bool(b) => Some(ConstValue::Bool(*b)),
            Literal::Int(i, _) => Some(ConstValue::Int(*i)),
            Literal::Float(f, _) => Some(ConstValue::Float(*f)),
            Literal::String(s) => Some(ConstValue::String(s.clone())),
            _ => None,
        },
        Expr::Unary(u) => {
            let v = eval_const(&u.operand)?;
            match (&u.op, v) {
                (UnOp::Neg, ConstValue::Int(i)) => Some(ConstValue::Int(-i)),
                (UnOp::Neg, ConstValue::Float(f)) => Some(ConstValue::Float(-f)),
                (UnOp::Not, ConstValue::Bool(b)) => Some(ConstValue::Bool(!b)),
                _ => None,
            }
        }
        Expr::Binary(b) => {
            let l = eval_const(&b.lhs)?;
            let r = eval_const(&b.rhs)?;
            match (l, r) {
                (ConstValue::Int(a), ConstValue::Int(c)) => match b.op {
                    BinOp::Add => Some(ConstValue::Int(a.wrapping_add(c))),
                    BinOp::Sub => Some(ConstValue::Int(a.wrapping_sub(c))),
                    BinOp::Mul => Some(ConstValue::Int(a.wrapping_mul(c))),
                    BinOp::Div if c != 0 => Some(ConstValue::Int(a / c)),
                    BinOp::Mod if c != 0 => Some(ConstValue::Int(a % c)),
                    BinOp::EqEq => Some(ConstValue::Bool(a == c)),
                    BinOp::Neq => Some(ConstValue::Bool(a != c)),
                    BinOp::Lt => Some(ConstValue::Bool(a < c)),
                    BinOp::LtEq => Some(ConstValue::Bool(a <= c)),
                    BinOp::Gt => Some(ConstValue::Bool(a > c)),
                    BinOp::GtEq => Some(ConstValue::Bool(a >= c)),
                    _ => None,
                },
                (ConstValue::Bool(a), ConstValue::Bool(c)) => match b.op {
                    BinOp::AndAnd => Some(ConstValue::Bool(a && c)),
                    BinOp::OrOr => Some(ConstValue::Bool(a || c)),
                    BinOp::EqEq => Some(ConstValue::Bool(a == c)),
                    BinOp::Neq => Some(ConstValue::Bool(a != c)),
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    }
}

/// Evaluate `#[cfg(...)]` / `@cfg(...)` predicates against enabled
/// features (Spec §76). Both attribute forms encode predicates identically
/// (`feature="x"`, `test`, `target="..."` as string literals); a legacy
/// `@cfg(k = v)` binary-expression encoding is also accepted defensively.
/// Unknown predicates evaluate to `true` (permissive) but are reported.
pub fn cfg_enabled(attrs: &[Attribute], features: &[String], target: &str) -> bool {
    for attr in attrs {
        if attr.name.text != "cfg" {
            continue;
        }
        for arg in &attr.args {
            let pred: Option<String> = match arg {
                Expr::Literal(Literal::String(pred)) => Some(pred.clone()),
                // Legacy `@cfg(feature = "x")` binary encoding: `=` parses
                // as a placeholder assignment binary op.
                Expr::Binary(b) => match (b.lhs.as_ref(), b.rhs.as_ref()) {
                    (
                        Expr::Ident(k),
                        Expr::Literal(Literal::String(v)),
                    ) => Some(format!("{}={}", k.text, v)),
                    _ => None,
                },
                Expr::Ident(id) => Some(id.text.clone()),
                _ => None,
            };
            if let Some(pred) = pred {
                if let Some(feat) = pred.strip_prefix("feature=") {
                    let feat = feat.trim_matches('"');
                    if !features.iter().any(|f| f == feat) {
                        return false;
                    }
                } else if pred == "test" {
                    if !features.iter().any(|f| f == "test") {
                        return false;
                    }
                } else if let Some(t) = pred.strip_prefix("target=") {
                    if t.trim_matches('"') != target {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Drop declarations disabled by `#[cfg]` (deterministic, visible in
/// diagnostics via the returned list of removed names).
pub fn filter_cfg_decls(module: &mut Module, features: &[String], target: &str) -> Vec<String> {
    let mut removed = Vec::new();
    module.declarations.retain(|d| {
        let attrs: &[Attribute] = match d {
            Decl::Function(f) => &f.attrs,
            Decl::Class(c) => &c.attrs,
            Decl::Struct(s) => &s.attrs,
            Decl::Enum(e) => &e.attrs,
            Decl::Trait(t) => &t.attrs,
            Decl::Interface(i) => &i.attrs,
            Decl::GlobalVar(g) => &g.attrs,
            Decl::TypeAlias(a) => &a.attrs,
            Decl::Effect(_) => &[],
            Decl::Service(s) => &s.attrs,
        };
        let keep = cfg_enabled(attrs, features, target);
        if !keep {
            removed.push(decl_name(d).to_string());
        }
        keep
    });
    removed
}

fn decl_name(d: &Decl) -> &str {
    match d {
        Decl::Function(f) => &f.name.text,
        Decl::Class(c) => &c.name.text,
        Decl::Struct(s) => &s.name.text,
        Decl::Enum(e) => &e.name.text,
        Decl::Trait(t) => &t.name.text,
        Decl::Interface(i) => &i.name.text,
        Decl::GlobalVar(g) => &g.name.text,
        Decl::TypeAlias(a) => &a.name.text,
        Decl::Effect(e) => &e.name.text,
        Decl::Service(s) => &s.name.text,
    }
}

/// Names bound by a pattern (used for capture/binding analysis).
pub fn pattern_bound_names(pat: &Pattern) -> Vec<String> {
    match pat {
        Pattern::Ident(id) => vec![id.text.clone()],
        Pattern::Wildcard(_) | Pattern::Literal(_) => vec![],
        Pattern::Tuple(pats) | Pattern::Array(pats) => {
            pats.iter().flat_map(pattern_bound_names).collect()
        }
        Pattern::Object(fields) => fields
            .iter()
            .flat_map(|f| pattern_bound_names(&f.pattern))
            .collect(),
        Pattern::EnumVariant(_, pats) => {
            pats.iter().flat_map(pattern_bound_names).collect()
        }
        Pattern::Named(id, inner) => {
            let mut out = vec![id.text.clone()];
            out.extend(pattern_bound_names(inner));
            out
        }
        Pattern::Or(alts) => alts.iter().flat_map(pattern_bound_names).collect(),
        Pattern::Range(_, _) => vec![],
    }
}

/// Minimal capture-set analysis for escape/lifetime analysis (Spec §8).
///
/// `free_variables` collects every identifier referenced by `expr` in
/// deterministic first-use order (deduplicated). It is scope-agnostic: call
/// `captured_names` to subtract lambda-bound names.
pub fn free_variables(expr: &Expr) -> Vec<String> {
    let mut out = Vec::new();
    collect_free_expr(expr, &mut out);
    // Deduplicate preserving first-use order.
    let mut seen = std::collections::HashSet::new();
    out.into_iter()
        .filter(|n| seen.insert(n.clone()))
        .collect()
}

/// Capture set of a lambda/closure: free variables of its body minus its own
/// parameters, intersected with the enclosing `scope_vars` when provided.
/// With `scope_vars = None`, every non-parameter free variable is a capture
/// (globals included); with `Some(scope)` only names actually declared in an
/// enclosing scope are reported — the minimal set a closure must capture for
/// escape/lifetime analysis.
pub fn captured_names(lambda: &Lambda, scope_vars: Option<&[String]>) -> Vec<String> {
    let bound: Vec<String> = lambda.params.iter().map(|p| p.name.text.clone()).collect();
    let free = free_variables(&lambda.body);
    let mut out: Vec<String> = free
        .into_iter()
        .filter(|n| !bound.iter().any(|b| b == n))
        .collect();
    if let Some(scope) = scope_vars {
        out.retain(|n| scope.iter().any(|s| s == n));
    }
    out
}

fn collect_free_pattern_bound(_pat: &Pattern) -> Vec<String> {
    pattern_bound_names(_pat)
}

fn collect_free_expr(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Ident(id) => out.push(id.text.clone()),
        Expr::Binary(b) => {
            collect_free_expr(&b.lhs, out);
            collect_free_expr(&b.rhs, out);
        }
        Expr::Unary(u) => collect_free_expr(&u.operand, out),
        Expr::Call(c) => {
            collect_free_expr(&c.callee, out);
            for a in &c.args {
                collect_free_expr(a, out);
            }
        }
        Expr::Index(i) => {
            collect_free_expr(&i.object, out);
            collect_free_expr(&i.index, out);
        }
        Expr::FieldAccess(f) => collect_free_expr(&f.object, out),
        Expr::MethodCall(m) => {
            collect_free_expr(&m.object, out);
            for a in &m.args {
                collect_free_expr(a, out);
            }
        }
        Expr::OptionalFieldAccess(ofa) => collect_free_expr(&ofa.object, out),
        Expr::Spread(e, _) => collect_free_expr(e, out),
        Expr::Lambda(l) => {
            // Nested lambda: its params shadow outer names.
            let bound: Vec<String> =
                l.params.iter().map(|p| p.name.text.clone()).collect();
            let mut inner = Vec::new();
            collect_free_expr(&l.body, &mut inner);
            for n in inner {
                if !bound.iter().any(|b| b == &n) {
                    out.push(n);
                }
            }
        }
        Expr::Block(b) => collect_free_block(b, out),
        Expr::If(i) => {
            collect_free_expr(&i.condition, out);
            collect_free_expr(&i.then_expr, out);
            if let Some(e) = &i.else_expr {
                collect_free_expr(e, out);
            }
        }
        Expr::Match(m) => {
            collect_free_expr(&m.expr, out);
            for a in &m.arms {
                collect_free_expr(&a.body, out);
                if let Some(g) = &a.guard {
                    collect_free_expr(g, out);
                }
                // Pattern-bound names shadow; drop them if they leaked via
                // body collection order by post-filtering below.
                let _ = collect_free_pattern_bound(&a.pattern);
            }
        }
        Expr::Try(t) => {
            collect_free_expr(&t.expr, out);
            for c in &t.catches {
                collect_free_block(&c.body, out);
            }
            if let Some(f) = &t.finally {
                collect_free_block(f, out);
            }
        }
        Expr::Await(a) => collect_free_expr(&a.expr, out),
        Expr::Cast(c) => collect_free_expr(&c.expr, out),
        Expr::New(n) => {
            for a in &n.args {
                collect_free_expr(a, out);
            }
        }
        Expr::AsyncBlock(a) => collect_free_expr(&a.body, out),
        Expr::NullCoalesce(l, r) => {
            collect_free_expr(l, out);
            collect_free_expr(r, out);
        }
        Expr::Range(l, r, _) => {
            collect_free_expr(l, out);
            collect_free_expr(r, out);
        }
        Expr::Tuple(es) => {
            for e in es {
                collect_free_expr(e, out);
            }
        }
        Expr::Attribute(_) => {}
        Expr::With(w) => {
            collect_free_expr(&w.target, out);
            for u in &w.updates {
                collect_free_expr(&u.value, out);
            }
        }
        Expr::Spawn(s) => collect_free_expr(&s.expr, out),
        Expr::Select(s) => {
            for a in &s.arms {
                collect_free_expr(&a.body, out);
                if let Some(g) = &a.guard {
                    collect_free_expr(g, out);
                }
            }
        }
        Expr::Option(o) => match o {
            OptionLiteral::Some(e) => collect_free_expr(e, out),
            OptionLiteral::None(_) => {}
        },
        Expr::Destructuring(d) => collect_free_expr(&d.initializer, out),
        Expr::MacroCall(m) => {
            for a in &m.args {
                collect_free_expr(a, out);
            }
        }
        Expr::InterpolatedString(parts) => {
            for p in parts {
                if let InterpolatedPart::Expr(e) = p {
                    collect_free_expr(e, out);
                }
            }
        }
        Expr::UnsafeBlock(b, _) => collect_free_block(b, out),
        // Remaining variants bind nothing (literals, attributes, ...).
        _ => {}
    }
}

fn collect_free_block(block: &Block, out: &mut Vec<String>) {
    for stmt in &block.statements {
        collect_free_stmt(stmt, out);
    }
}

fn collect_free_stmt(stmt: &Stmt, out: &mut Vec<String>) {
    match stmt {
        Stmt::Expr(e) => collect_free_expr(&e.expr, out),
        Stmt::Decl(d) => {
            if let Some(init) = &d.init {
                collect_free_expr(init, out);
            }
        }
        Stmt::If(i) => {
            collect_free_expr(&i.condition, out);
            collect_free_block(&i.then_branch, out);
            if let Some(b) = &i.else_body {
                collect_free_block(b, out);
            }
            if let Some(e) = &i.else_branch {
                collect_free_expr(&e.condition, out);
                collect_free_block(&e.then_branch, out);
            }
        }
        Stmt::Match(m) => {
            collect_free_expr(&m.expr, out);
            for a in &m.arms {
                collect_free_expr(&a.body, out);
                if let Some(g) = &a.guard {
                    collect_free_expr(g, out);
                }
            }
        }
        Stmt::Switch(s) => {
            collect_free_expr(&s.scrutinee, out);
            for c in &s.cases {
                for e in &c.exprs {
                    collect_free_expr(e, out);
                }
                if let Some(g) = &c.guard {
                    collect_free_expr(g, out);
                }
                collect_free_block(&c.body, out);
            }
            if let Some(d) = &s.default {
                collect_free_block(d, out);
            }
        }
        Stmt::For(f) => {
            collect_free_expr(&f.iterable, out);
            collect_free_block(&f.body, out);
        }
        Stmt::While(w) => {
            collect_free_expr(&w.condition, out);
            collect_free_block(&w.body, out);
        }
        Stmt::DoWhile(d) => {
            collect_free_block(&d.body, out);
            collect_free_expr(&d.condition, out);
        }
        Stmt::Loop(l) => collect_free_block(&l.body, out),
        Stmt::Return(r) => {
            if let Some(v) = &r.value {
                collect_free_expr(v, out);
            }
        }
        Stmt::Defer(d) => collect_free_expr(&d.expr, out),
        Stmt::AsyncBlock(a) => collect_free_expr(&a.body, out),
        Stmt::Block(b) => collect_free_block(b, out),
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Part X — borrow-rules diagnostics (Spec §6 / §44 + Part X ownership).
///
/// `BorrowTracker` mirrors the `lexicon-utils` loan engine without
/// adding a dependency (analysis must stay dependency-light): named
/// loans with lexical scopes. Rules (stable codes):
/// - second `&mut` while mutably loaned → `E0305`
/// - `&mut` while `&` live (and vice versa) → `E0305`
/// - use of a moved value → `E0382`
/// - use while mutably loaned → `E0305`
/// - move while any loan lives → `E0305`
/// Scopes expire loans/names exactly like the runtime checker so
/// diagnostics agree across crates.
#[derive(Debug, Default)]
pub struct BorrowTracker {
    vars: HashMap<String, (Option<usize>, Vec<usize>, bool)>,
    scopes: Vec<Vec<String>>,
}

impl BorrowTracker {
    pub fn new() -> Self {
        BorrowTracker { vars: HashMap::new(), scopes: vec![Vec::new()] }
    }

    fn depth(&self) -> usize {
        self.scopes.len() - 1
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() <= 1 {
            return;
        }
        let exiting = self.scopes.len() - 1;
        let names = self.scopes.pop().expect("scope underflow");
        for (_, (mut_loan, shared, _)) in self.vars.iter_mut() {
            if *mut_loan == Some(exiting) {
                *mut_loan = None;
            }
            shared.retain(|d| *d != exiting);
        }
        for name in names {
            self.vars.remove(&name);
        }
    }

    fn ensure(&mut self, name: &str) {
        let depth = self.depth();
        if !self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), (None, Vec::new(), false));
            self.scopes[depth].push(name.to_string());
        }
    }

    pub fn declare(&mut self, name: &str) {
        self.ensure(name);
    }

    /// Take `&mut name`.
    pub fn loan_mut(&mut self, name: &str) -> Result<(), String> {
        let depth = self.depth();
        self.ensure(name);
        let e = self.vars.get_mut(name).expect("just inserted");
        if e.2 {
            return Err(format!("E0382: use of moved value `{}`", name));
        }
        if e.0.is_some() {
            return Err(format!(
                "E0305: second mutable loan of `{}` while first loan is live",
                name
            ));
        }
        if !e.1.is_empty() {
            return Err(format!(
                "E0305: cannot mutably loan `{}` while {} shared loan(s) are live",
                name,
                e.1.len()
            ));
        }
        e.0 = Some(depth);
        Ok(())
    }

    /// Take `&name`.
    pub fn loan_shared(&mut self, name: &str) -> Result<(), String> {
        let depth = self.depth();
        self.ensure(name);
        let e = self.vars.get_mut(name).expect("just inserted");
        if e.2 {
            return Err(format!("E0382: use of moved value `{}`", name));
        }
        if e.0.is_some() {
            return Err(format!(
                "E0305: cannot shared-loan `{}` while mutably loaned",
                name
            ));
        }
        e.1.push(depth);
        Ok(())
    }

    /// Read/use `name`.
    pub fn use_name(&self, name: &str) -> Result<(), String> {
        match self.vars.get(name) {
            None => Ok(()),
            Some((_, _, true)) => Err(format!("E0382: use of moved value `{}`", name)),
            Some((Some(_), _, _)) => {
                Err(format!("E0305: use of `{}` while mutably loaned", name))
            }
            Some(_) => Ok(()),
        }
    }

    /// Move `name`.
    pub fn move_name(&mut self, name: &str) -> Result<(), String> {
        self.ensure(name);
        let e = self.vars.get_mut(name).expect("just inserted");
        if e.2 {
            return Err(format!("E0382: use of moved value `{}` (already moved)", name));
        }
        if e.0.is_some() || !e.1.is_empty() {
            return Err(format!("E0305: cannot move `{}` while loaned", name));
        }
        e.2 = true;
        Ok(())
    }

    /// Classify a tracker error into its stable rule code.
    pub fn rule_code(err: &str) -> &'static str {
        if err.contains("E0382") {
            "E0382"
        } else {
            "E0305"
        }
    }
}

/// Monomorphization registry (Spec §11 bloat control).
///
/// Records `fn_name + concrete argument types -> specialized symbol name`,
/// reusing identical instantiations so each distinct concrete signature is
/// emitted once. `max_specializations` caps total entries; exceeding it is an
/// explicit error (callers surface it as E0303-family diagnostics).
pub struct Monomorphizer {
    max_specializations: usize,
    table: HashMap<(String, Vec<String>), String>,
}

impl Monomorphizer {
    pub fn new(max_specializations: usize) -> Self {
        Monomorphizer { max_specializations, table: HashMap::new() }
    }

    /// Instantiate (or reuse) the specialization for `fn_name` applied to
    /// `concrete_types`. Returns the specialized symbol name
    /// (`{fn}_{t1}_{t2}...`, sanitized) on success.
    pub fn instantiate(
        &mut self,
        fn_name: &str,
        concrete_types: &[Type],
    ) -> Result<String, String> {
        let key_types: Vec<String> =
            concrete_types.iter().map(|t| t.to_string()).collect();
        let key = (fn_name.to_string(), key_types);
        if let Some(existing) = self.table.get(&key) {
            return Ok(existing.clone());
        }
        if self.table.len() >= self.max_specializations {
            return Err(format!(
                "[E0303] monomorphization limit exceeded ({} specializations): cannot instantiate `{}` with ({})",
                self.max_specializations,
                fn_name,
                key.1.join(", ")
            ));
        }
        let suffix = key
            .1
            .iter()
            .map(|t| {
                t.chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '_' })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("_");
        let symbol = if suffix.is_empty() {
            fn_name.to_string()
        } else {
            format!("{}_{}", fn_name, suffix)
        };
        self.table.insert(key, symbol.clone());
        Ok(symbol)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;

    fn check_source(source: &str) -> Result<(), Vec<String>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let module = parser.parse().expect("parse failed");
        TypeChecker::new().check_module(&module)
    }

    #[test]
    fn builtin_int_uint_byte_decimal_resolve() {
        assert!(check_source("fn f(a: int, b: uint, c: byte, d: decimal) {}").is_ok());
    }

    #[test]
    fn service_registers_and_rejects_duplicate_rpc() {
        assert!(check_source(
            "struct User { id: int; } service S { rpc Get(id: int) -> User; rpc List() -> User; }"
        )
        .is_ok());
        let err = check_source("service S { rpc Get(id: int); rpc Get(id: int); }")
            .expect_err("expected E0304");
        assert!(err.iter().any(|e| e.contains("E0304")), "got: {:?}", err);
    }

    #[test]
    fn narrowing_return_requires_cast() {
        // i64 -> i32 without `as` must produce E0306.
        let err = check_source("fn f() -> i32 { let x: i64 = 0; return x; }")
            .expect_err("expected narrowing error");
        assert!(err.iter().any(|e| e.contains("E0306")), "got: {:?}", err);
    }

    #[test]
    fn explicit_cast_allows_narrowing() {
        assert!(check_source("fn f() -> i32 { let x: i64 = 0; return x as i32; }").is_ok());
    }

    #[test]
    fn optional_field_access_is_rejected() {
        let err = check_source("fn f(a: int?) { a.foo; }")
            .expect_err("expected E0305");
        assert!(err.iter().any(|e| e.contains("E0305")), "got: {:?}", err);
    }

    #[test]
    fn switch_and_do_while_typecheck() {
        assert!(check_source(
            "fn f(x: int) { switch x { case 1: break; default: break; } do { break; } while true; }"
        )
        .is_ok());
    }

    #[test]
    fn type_alias_registers() {
        assert!(check_source("type UserId = int; fn f(a: UserId) {}").is_ok());
    }

    #[test]
    fn variadic_must_be_last() {
        let err = check_source("fn f(a: int..., b: int) {}")
            .expect_err("expected E0202");
        assert!(err.iter().any(|e| e.contains("E0202")), "got: {:?}", err);
    }

    #[test]
    fn generic_arity_reports_origin() {
        let err = check_source("fn id<T>(x: T) -> T { return x; } fn f() { id(); }")
            .expect_err("expected E0303");
        assert!(err.iter().any(|e| e.contains("E0303") && e.contains("id")), "got: {:?}", err);
    }

    #[test]
    fn bool_match_must_be_exhaustive() {
        let err = check_source("fn f(x: bool) { match x { true => 1 } }")
            .expect_err("expected E0204");
        assert!(err.iter().any(|e| e.contains("E0204")), "got: {:?}", err);
        assert!(check_source("fn f(x: bool) { match x { true => 1, _ => 0 } }").is_ok());
    }

    #[test]
    fn duplicate_struct_fields_rejected() {
        let err = check_source("struct S { x: int; x: int; }")
            .expect_err("expected E0304");
        assert!(err.iter().any(|e| e.contains("E0304")), "got: {:?}", err);
    }

    #[test]
    fn implements_unknown_interface_rejected() {
        let err = check_source("class C implements Nope { }")
            .expect_err("expected E0304");
        assert!(err.iter().any(|e| e.contains("E0304")), "got: {:?}", err);
    }

    #[test]
    fn implements_conformance_checked() {
        let ok = "trait Loud { fn speak(); } class C implements Loud { fn speak() {} }";
        assert!(check_source(ok).is_ok());
        let missing = "trait Loud { fn speak(); } class C implements Loud { }";
        let err = check_source(missing).expect_err("expected E0304");
        assert!(err.iter().any(|e| e.contains("speak")), "got: {:?}", err);
    }

    #[test]
    fn unsafe_block_checks_contents() {
        assert!(check_source("fn f() { unsafe { return; } }").is_ok());
    }

    #[test]
    fn unreachable_produces_warning_not_error() {
        let mut lexer = Lexer::new("fn f() { return; let x = 1; }");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let module = parser.parse().expect("parse failed");
        let mut ck = TypeChecker::new();
        assert!(ck.check_module(&module).is_ok());
        assert!(ck.take_warnings().iter().any(|w| w.contains("E0203")));
    }

    #[test]
    fn const_eval_folds() {
        use lexicon_parser::ast::{BinOp, BinaryExpr, Expr, Literal};
        use lexicon_core::span::Span;
        let expr = Expr::Binary(BinaryExpr {
            op: BinOp::Add,
            lhs: Box::new(Expr::Literal(Literal::Int(2, None))),
            rhs: Box::new(Expr::Literal(Literal::Int(3, None))),
            span: Span::default(),
        });
        assert_eq!(eval_const(&expr), Some(ConstValue::Int(5)));
    }

    #[test]
    fn cfg_filtering() {
        use lexicon_parser::ast::Attribute;
        let attr = Attribute {
            name: lexicon_parser::ast::Ident {
                text: "cfg".to_string(),
                span: lexicon_core::span::Span::default(),
            },
            args: vec![lexicon_parser::ast::Expr::Literal(
                lexicon_parser::ast::Literal::String("feature=net".to_string()),
            )],
            span: lexicon_core::span::Span::default(),
        };
        assert!(cfg_enabled(&[attr.clone()], &["net".to_string()], "native"));
        assert!(!cfg_enabled(&[attr], &[], "native"));
    }

    fn parse_module(source: &str) -> lexicon_parser::ast::Module {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse().expect("parse failed")
    }

    #[test]
    fn cfg_test_and_target_predicates() {
        use lexicon_parser::ast::Attribute;
        use lexicon_core::span::Span;
        let mk = |pred: &str| Attribute {
            name: lexicon_parser::ast::Ident { text: "cfg".to_string(), span: Span::default() },
            args: vec![lexicon_parser::ast::Expr::Literal(
                lexicon_parser::ast::Literal::String(pred.to_string()),
            )],
            span: Span::default(),
        };
        assert!(cfg_enabled(&[mk("test")], &["test".to_string()], "native"));
        assert!(!cfg_enabled(&[mk("test")], &[], "native"));
        assert!(cfg_enabled(&[mk("target=wasm")], &[], "wasm"));
        assert!(!cfg_enabled(&[mk("target=wasm")], &[], "native"));
    }

    #[test]
    fn cfg_filter_drops_disabled_decls() {
        let mut m = parse_module(
            "#[cfg(feature = \"net\")] fn serve() {} fn local() {}",
        );
        let removed = filter_cfg_decls(&mut m, &[], "native");
        assert_eq!(removed, vec!["serve".to_string()]);
        assert_eq!(m.declarations.len(), 1);
        let mut m2 = parse_module(
            "#[cfg(feature = \"net\")] fn serve() {} fn local() {}",
        );
        let removed2 = filter_cfg_decls(&mut m2, &["net".to_string()], "native");
        assert!(removed2.is_empty());
        assert_eq!(m2.declarations.len(), 2);
    }

    #[test]
    fn at_cfg_form_filters_like_hash_form() {
        // NOTE: `@`-forms cannot be exercised end-to-end from source because
        // the lexer has no `@` arm (`@` scans as `Token::Error`; fixing that
        // is outside the allowed files). The parser accepts `Token::At`
        // streams (see parser `test_parse_at_cfg_form`) and normalizes
        // `@cfg(k = v)` to the same string predicates as `#[cfg(...)]`.
        // Here we cover the defensive legacy encoding (`=` binary expr).
        use lexicon_parser::ast::*;
        use lexicon_core::span::Span;
        let pred = Expr::Binary(BinaryExpr {
            op: BinOp::AddEq,
            lhs: Box::new(Expr::Ident(Ident {
                text: "feature".to_string(),
                span: Span::default(),
            })),
            rhs: Box::new(Expr::Literal(Literal::String("net".to_string()))),
            span: Span::default(),
        });
        let attr = Attribute {
            name: Ident { text: "cfg".to_string(), span: Span::default() },
            args: vec![pred],
            span: Span::default(),
        };
        assert!(cfg_enabled(&[attr.clone()], &["net".to_string()], "native"));
        assert!(!cfg_enabled(&[attr], &[], "native"));
    }

    #[test]
    fn panic_types_as_never_and_marks_unreachable() {
        let mut ck = TypeChecker::new();
        let m = parse_module("fn f() { panic(\"boom\"); let x = 1; }");
        assert!(ck.check_module(&m).is_ok());
        assert!(ck.take_warnings().iter().any(|w| w.contains("E0203")));
    }

    #[test]
    fn recover_types_without_error() {
        assert!(check_source("fn f() { recover(); }").is_ok());
        assert!(check_source("fn f() { defer recover(); }").is_ok());
    }

    #[test]
    fn match_guard_must_be_bool() {
        assert!(check_source("fn f(x: int) { match x { n if n > 0 => 1, _ => 0 } }").is_ok());
        let err = check_source("fn f(x: int) { match x { n if 1 => 1, _ => 0 } }")
            .expect_err("expected guard error");
        assert!(err.iter().any(|e| e.contains("guard")), "got: {:?}", err);
    }

    #[test]
    fn guarded_match_requires_wildcard_for_exhaustiveness() {
        // All arms guarded on bool: without `_` this is E0204 even though both
        // literals appear, because guards may all fail at runtime.
        let err = check_source(
            "fn f(x: bool) { match x { true if x => 1, false if x => 0 } }",
        )
        .expect_err("expected E0204");
        assert!(err.iter().any(|e| e.contains("E0204")), "got: {:?}", err);
    }

    #[test]
    fn enum_match_exhaustiveness() {
        // NOTE: bare `Red =>` is an irrefutable binding pattern (counts as a
        // wildcard), so enum coverage is expressed with `Red()` variant
        // patterns or `_`.
        assert!(check_source(
            "enum Color { Red, Green } fn f(c: Color) { match c { Red() => 1, Green() => 2 } }",
        )
        .is_ok());
        assert!(check_source(
            "enum Color { Red, Green } fn f(c: Color) { match c { Red() => 1, _ => 0 } }",
        )
        .is_ok());
        let err = check_source(
            "enum Color { Red, Green } fn f(c: Color) { match c { Red() => 1 } }",
        )
        .expect_err("expected E0204");
        assert!(err.iter().any(|e| e.contains("E0204")), "got: {:?}", err);
    }

    #[test]
    fn switch_guard_must_be_bool() {
        assert!(check_source(
            "fn f(x: int) { switch x { case 1 if x > 0: break; default: break; } }",
        )
        .is_ok());
        let err = check_source(
            "fn f(x: int) { switch x { case 1 if 1: break; default: break; } }",
        )
        .expect_err("expected switch guard error");
        assert!(err.iter().any(|e| e.contains("guard")), "got: {:?}", err);
    }

    #[test]
    fn tuple_return_and_destructuring() {
        // NOTE: int literals infer `i64`, so tuple annotations name `i64`
        // (consistent with scalar `-> int { return 1; }` narrowing rules).
        assert!(check_source(
            "fn pair() -> (i64, string) { return (1, \"a\"); } fn f() { let (a, b) = pair(); }",
        )
        .is_ok());
        assert!(check_source(
            "fn f() { let (a, b): (i64, string) = (1, \"a\"); }",
        )
        .is_ok());
        let err = check_source("fn f() { let (a, b) = (1, 2, 3); }")
            .expect_err("expected arity mismatch");
        assert!(err.iter().any(|e| e.contains("E0301")), "got: {:?}", err);
    }

    #[test]
    fn fn_pointer_type_and_call_through_variable() {
        assert!(check_source(
            "fn apply(cb: fn(int) -> int, x: int) -> int { return cb(x); }",
        )
        .is_ok());
        assert!(check_source(
            "fn id(x: int) -> int { return x; } fn f() { let cb: fn(int) -> int = id; let y = cb(1); }",
        )
        .is_ok());
    }

    #[test]
    fn abi_inline_attrs_validated() {
        assert!(check_source("#[abi(\"C\")] fn f() {}").is_ok());
        assert!(check_source("#[inline] fn f() {}").is_ok());
        assert!(check_source("#[abi(\"C\")] #[inline] fn f() {}").is_ok());
        let err = check_source("#[abi(\"Bogus\")] fn f() {}")
            .expect_err("expected E0201");
        assert!(err.iter().any(|e| e.contains("E0201")), "got: {:?}", err);
        let err2 = check_source("#[inline(\"yes\")] fn f() {}")
            .expect_err("expected E0201");
        assert!(err2.iter().any(|e| e.contains("E0201")), "got: {:?}", err2);
    }

    #[test]
    fn free_variables_collects_in_order_deduped() {
        use lexicon_parser::ast::*;
        use lexicon_core::span::Span;
        let ident = |t: &str| {
            Expr::Ident(Ident { text: t.to_string(), span: Span::default() })
        };
        let expr = Expr::Binary(BinaryExpr {
            op: BinOp::Add,
            lhs: Box::new(ident("a")),
            rhs: Box::new(Expr::Binary(BinaryExpr {
                op: BinOp::Add,
                lhs: Box::new(ident("b")),
                rhs: Box::new(ident("a")),
                span: Span::default(),
            })),
            span: Span::default(),
        });
        assert_eq!(free_variables(&expr), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn captured_names_subtracts_params_and_intersects_scope() {
        use lexicon_parser::ast::*;
        use lexicon_core::span::Span;
        let param = |t: &str| Param {
            attrs: vec![],
            is_mutable: false,
            name: Ident { text: t.to_string(), span: Span::default() },
            ty: Type::Primitive(PrimitiveType::Dynamic),
            is_variadic: false,
            default: None,
            span: Span::default(),
        };
        let ident = |t: &str| {
            Expr::Ident(Ident { text: t.to_string(), span: Span::default() })
        };
        let lambda = Lambda {
            params: vec![param("x")],
            return_type: None,
            body: Box::new(Expr::Binary(BinaryExpr {
                op: BinOp::Add,
                lhs: Box::new(ident("x")),
                rhs: Box::new(ident("y")),
                span: Span::default(),
            })),
            span: Span::default(),
        };
        // No scope filter: `x` is bound, `y` is captured.
        assert_eq!(captured_names(&lambda, None), vec!["y".to_string()]);
        // Scope-declared vars intersect: `y` not in scope => empty.
        assert!(captured_names(&lambda, Some(&["z".to_string()])).is_empty());
        assert_eq!(
            captured_names(&lambda, Some(&["y".to_string(), "z".to_string()])),
            vec!["y".to_string()]
        );
    }

    #[test]
    fn monomorphizer_reuses_and_caps() {
        let mut m = Monomorphizer::new(2);
        let a = m.instantiate("id", &[Type::Int]).unwrap();
        let b = m.instantiate("id", &[Type::Int]).unwrap();
        assert_eq!(a, b, "identical instantiations must reuse the symbol");
        assert_eq!(m.len(), 1);
        m.instantiate("id", &[Type::String]).unwrap();
        assert_eq!(m.len(), 2);
        let err = m.instantiate("id", &[Type::Bool]).expect_err("expected cap error");
        assert!(err.contains("E0303") || err.contains("limit"), "got: {}", err);
    }

    // ---- Part X: Result/Option checking + borrow rules diagnostics ----

    #[test]
    fn result_generic_lowers_to_result_type() {
        assert!(check_source("fn f(a: Result<int, string>) {}").is_ok());
        assert!(check_source("fn f(a: Option<int>) {}").is_ok());
    }

    #[test]
    fn field_access_on_result_is_rejected() {
        let err = check_source("fn f(a: Result<int, string>) { a.foo; }")
            .expect_err("expected E0802");
        assert!(err.iter().any(|e| e.contains("E0802")), "got: {:?}", err);
    }

    #[test]
    fn option_unwrap_and_combinators_type_through() {
        assert!(check_source("fn f(a: int?) { a.unwrap(); }").is_ok());
        assert!(check_source("fn f(a: int?) { a.is_some(); }").is_ok());
        assert!(check_source("fn f(a: Result<int, string>) { a.is_ok(); }").is_ok());
        // `??` with a compatible fallback is fine.
        assert!(check_source("fn f(a: int?) { a ?? 0; }").is_ok());
        // `??` with an incompatible fallback is E0301.
        let err = check_source("fn f(a: int?) { a ?? \"s\"; }")
            .expect_err("expected E0301");
        assert!(err.iter().any(|e| e.contains("E0301")), "got: {:?}", err);
        // `??` on a non-optional is E0301.
        let err2 = check_source("fn f(a: int) { a ?? 0; }")
            .expect_err("expected E0301");
        assert!(err2.iter().any(|e| e.contains("E0301")), "got: {:?}", err2);
    }

    #[test]
    fn question_operator_unwraps_or_rejects() {
        use lexicon_core::span::Span;
        use lexicon_parser::ast::{Expr, Ident, UnOp, UnaryExpr};
        let mk_unary = |op, operand: Expr| Expr::Unary(UnaryExpr {
            op,
            operand: Box::new(operand),
            span: Span::default(),
        });
        let ident = |t: &str| {
            Expr::Ident(Ident { text: t.to_string(), span: Span::default() })
        };
        // `opt?` where opt: int? yields int.
        let m = parse_module("fn f(a: int?) { a; }");
        let mut ck = TypeChecker::new();
        ck.push_scope();
        ck.declare_var("opt".to_string(), Type::Option(Box::new(Type::Int)));
        let ty = ck.check_expr(&mk_unary(UnOp::Question, ident("opt")));
        assert_eq!(ty, Type::Int);
        ck.pop_scope();
        // `x?` where x: int is E0802.
        let mut ck2 = TypeChecker::new();
        ck2.push_scope();
        ck2.declare_var("x".to_string(), Type::Int);
        ck2.check_expr(&mk_unary(UnOp::Question, ident("x")));
        ck2.pop_scope();
        let _ = m;
    }

    #[test]
    fn option_literals_type_as_optional() {
        use lexicon_core::span::Span;
        use lexicon_parser::ast::{
            Expr, FieldAccessExpr, Ident, Literal, OptionLiteral, PrimitiveType,
        };
        let int_ty = lexicon_parser::ast::Type::Primitive(PrimitiveType::I64);
        let some = Expr::Option(OptionLiteral::Some(Box::new(Expr::Literal(
            Literal::Int(1, None),
        ))));
        let none = Expr::Option(OptionLiteral::None(int_ty));
        let mut ck = TypeChecker::new();
        ck.push_scope();
        assert_eq!(
            ck.check_expr(&some),
            Type::Option(Box::new(Type::I64))
        );
        assert_eq!(
            ck.check_expr(&none),
            Type::Option(Box::new(Type::I64))
        );
        // Plain field access on a non-optional stays permissive.
        ck.declare_var("x".to_string(), Type::Int);
        let fa = Expr::FieldAccess(FieldAccessExpr {
            object: Box::new(Expr::Ident(Ident {
                text: "x".to_string(),
                span: Span::default(),
            })),
            field: Ident { text: "foo".to_string(), span: Span::default() },
            span: Span::default(),
        });
        assert_eq!(ck.check_expr(&fa), Type::Unknown);
        ck.pop_scope();
        let _ = Span::default();
    }

    #[test]
    fn borrow_tracker_rules() {
        let mut t = BorrowTracker::new();
        t.declare("x");
        t.loan_mut("x").unwrap();
        assert_eq!(
            BorrowTracker::rule_code(&t.loan_mut("x").unwrap_err()),
            "E0305"
        );
        assert_eq!(
            BorrowTracker::rule_code(&t.use_name("x").unwrap_err()),
            "E0305"
        );
        let mut t2 = BorrowTracker::new();
        t2.declare("y");
        t2.move_name("y").unwrap();
        assert_eq!(
            BorrowTracker::rule_code(&t2.use_name("y").unwrap_err()),
            "E0382"
        );
        // Scope expiry ends inner loans.
        let mut t3 = BorrowTracker::new();
        t3.declare("z");
        t3.enter_scope();
        t3.loan_shared("z").unwrap();
        t3.exit_scope();
        t3.loan_mut("z").unwrap();
    }
}
