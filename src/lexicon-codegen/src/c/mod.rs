//! Native C backend for `lex run` / `lex build`.
//!
//! [`emit_program`] lowers a module to a single self-contained C translation
//! unit (gnu11) whose observable behavior matches the reference interpreter
//! (`lexicon-cli/src/interp.rs`) byte-for-byte: step accounting (50M budget),
//! output buffering + cap (1M bytes), display/debug/inspect rendering, Rust
//! display for f64, numeric semantics, and every runtime error string.
//!
//! Anything outside the supported subset returns `Err(reason)`; the CLI
//! treats that as "fall back to the interpreter", so unsupported programs
//! keep working — they just run at interpreter speed.

pub mod rt;

mod expr;
mod helpers;
mod infer;
mod stmt;

use lexicon_parser::ast::{Decl, Expr, Function, Module, PrimitiveType, Type};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use rt::RUNTIME;

pub fn emit_program(module: &Module) -> Result<String, String> {
    let mut em = Emitter::new(module)?;
    em.infer_returns()?;
    em.emit_module()?;
    em.expand_types()?;
    let (types, protos, bodies) = em.emit_helpers();
    Ok(em.finish(&types, &protos, &bodies))
}

/// Escape every byte that is not an ASCII letter/digit as `_XX` hex so any
/// source identifier becomes a valid, collision-free C identifier.
pub(crate) fn esc(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for b in name.bytes() {
        if b.is_ascii_alphanumeric() {
            out.push(b as char);
        } else {
            out.push_str(&format!("_{:02X}", b));
        }
    }
    out
}

/// A C string literal for `s` (escapes quotes/backslashes and every byte
/// outside printable ASCII as a 3-digit octal escape).
pub(crate) fn cstr_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for b in s.bytes() {
        match b {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7E => out.push(b as char),
            _ => out.push_str(&format!("\\{:03o}", b)),
        }
    }
    out.push('"');
    out
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Char,
    Unit,
    Null,
    List(Box<Ty>),
    Tuple(Vec<Ty>),
    Struct(String),
    Range,
    /// Statically-provable runtime error path (dead code): unifies with
    /// anything so downstream inference keeps working.
    Panic,
    /// Not resolved yet / unsupported. Poisons strict emission (Err).
    Unknown,
}

impl Ty {
    pub(crate) fn ctype(&self) -> String {
        match self {
            Ty::Int | Ty::Bool | Ty::Unit | Ty::Null | Ty::Panic | Ty::Unknown => {
                "long long".to_string()
            }
            Ty::Float => "double".to_string(),
            Ty::Str => "LxStr".to_string(),
            Ty::Char => "unsigned int".to_string(),
            Ty::List(t) => format!("LxL_{}", t.mangle()),
            Ty::Tuple(ts) => tuple_cname(ts),
            Ty::Struct(n) => format!("LxS_{}", esc(n)),
            Ty::Range => "LxRange".to_string(),
        }
    }

    pub(crate) fn mangle(&self) -> String {
        match self {
            Ty::Int => "it".to_string(),
            Ty::Float => "fl".to_string(),
            Ty::Bool => "bo".to_string(),
            Ty::Str => "st".to_string(),
            Ty::Char => "ch".to_string(),
            Ty::Unit => "un".to_string(),
            Ty::Null => "nu".to_string(),
            Ty::Panic => "pa".to_string(),
            Ty::Unknown => "uk".to_string(),
            Ty::List(t) => format!("l{}", t.mangle()),
            Ty::Tuple(ts) => format!("tu{}_{}", ts.len(), ts.iter().map(Ty::mangle).collect::<Vec<_>>().join("_")),
            Ty::Struct(n) => format!("s{}", esc(n)),
            Ty::Range => "ra".to_string(),
        }
    }

    /// `interp.rs::type_name` — used inside runtime error messages.
    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Ty::Int => "int",
            Ty::Float => "float",
            Ty::Bool => "bool",
            Ty::Str => "String",
            Ty::Char => "char",
            Ty::Unit => "void",
            Ty::Null => "null",
            Ty::List(_) => "List",
            Ty::Tuple(_) => "Tuple",
            Ty::Struct(_) => "struct",
            Ty::Range => "Range",
            Ty::Panic | Ty::Unknown => "",
        }
    }

    /// Whether reading a value of this type must deep-copy (the interpreter
    /// clones every `Val` on read; lists/structs/tuples are mutable in
    /// place, so C must not alias them across bindings).
    pub(crate) fn needs_clone(&self) -> bool {
        match self {
            Ty::List(_) | Ty::Struct(_) => true,
            Ty::Tuple(ts) => ts.iter().any(Ty::needs_clone),
            _ => false,
        }
    }

    pub(crate) fn is_num(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float)
    }
}

pub(crate) fn tuple_cname(ts: &[Ty]) -> String {
    format!(
        "LxT{}_{}",
        ts.len(),
        ts.iter().map(Ty::mangle).collect::<Vec<_>>().join("_")
    )
}

/// Unify two inferred types (same → same; `Panic` absorbs; `Unknown` yields
/// the other side; structurally-equal containers unify elementwise).
/// Anything else is a conflict → unsupported.
pub(crate) fn unify(a: &Ty, b: &Ty) -> Result<Ty, String> {
    if a == b {
        return Ok(a.clone());
    }
    match (a, b) {
        (Ty::Unknown, x) | (x, Ty::Unknown) => Ok(x.clone()),
        (Ty::Panic, x) | (x, Ty::Panic) => Ok(x.clone()),
        (Ty::List(x), Ty::List(y)) => Ok(Ty::List(Box::new(unify(x, y)?))),
        (Ty::Tuple(xs), Ty::Tuple(ys)) if xs.len() == ys.len() => {
            let mut out = Vec::with_capacity(xs.len());
            for (x, y) in xs.iter().zip(ys.iter()) {
                out.push(unify(x, y)?);
            }
            Ok(Ty::Tuple(out))
        }
        _ => Err(format!("type mismatch: {} vs {}", a.type_name(), b.type_name())),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
}

impl StructDef {
    pub(crate) fn find(&self, field: &str) -> Option<usize> {
        self.fields.iter().position(|(n, _)| n == field)
    }

    pub(crate) fn cname(&self) -> String {
        format!("LxS_{}", esc(&self.name))
    }
}

/// An emitted C expression: its text, its type, and whether evaluating it has
/// side effects beyond `lx_tick()` (calls, push/pop, output, panics). Effectful
/// sub-expressions are hoisted into temporaries so evaluation order matches the
/// interpreter's left-to-right order.
#[derive(Debug, Clone)]
pub(crate) struct ExprC {
    pub text: String,
    pub ty: Ty,
    pub eff: bool,
}

/// Where `break`/`continue` currently go. `Switch` inherits the enclosing
/// loop's targets (the interpreter propagates `Flow::Break` out of a switch);
/// with no enclosing loop both panic.
#[derive(Debug, Clone)]
pub(crate) struct LoopCtx {
    pub brk: Option<String>,
    pub cont: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Sig {
    pub cname: String,
    pub ret: Ty,
    pub params: Vec<(String, Ty)>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PeekCtx {
    pub rets: Vec<Ty>,
}

pub(crate) struct Emitter {
    // ---- module-level tables ----
    pub(crate) struct_decls: BTreeMap<String, Vec<(String, Type)>>,
    pub(crate) structs: BTreeMap<String, StructDef>,
    pub(crate) fns: BTreeMap<String, Function>,
    pub(crate) fn_order: Vec<String>,
    pub(crate) globals: Vec<(String, Type, Option<Expr>)>,
    pub(crate) global_tys: BTreeMap<String, Ty>,
    pub(crate) ret_tys: BTreeMap<String, Ty>,
    pub(crate) sigs: BTreeMap<String, Sig>,
    /// During global-initializer emission, how many `globals` are visible
    /// (the interpreter evaluates them in declaration order, so a later
    /// global is not yet bound).
    pub(crate) global_visible: usize,

    // ---- helper worklists (see helpers.rs) ----
    pub(crate) disp: BTreeSet<Ty>,
    pub(crate) dbg: BTreeSet<Ty>,
    pub(crate) insp: BTreeSet<Ty>,
    pub(crate) clones: BTreeSet<Ty>,
    pub(crate) eqs: BTreeSet<(Ty, Ty)>,
    pub(crate) lists: BTreeSet<Ty>,
    pub(crate) tuples: BTreeSet<Vec<Ty>>,
    pub(crate) used_structs: BTreeSet<String>,
    /// Guards `resolve_struct` against self-referential definitions.
    pub(crate) resolving: BTreeSet<String>,

    // ---- emission state ----
    /// Function bodies (and the pieces assembled in `finish`).
    pub(crate) out: String,
    pub(crate) indent: usize,
    /// Scope stack: name → (C variable, type). Each binding gets a unique C
    /// name, so shadowing a parameter or an outer local is safe.
    pub(crate) scopes: Vec<HashMap<String, (String, Ty)>>,
    /// Every name ever bound in the current function (union of all scopes):
    /// reading such a name outside its scope is an error rather than a stray
    /// C reference.
    pub(crate) bound: BTreeSet<String>,
    pub(crate) shadow_n: BTreeMap<String, usize>,
    pub(crate) loop_stack: Vec<LoopCtx>,
    pub(crate) label_n: usize,
    pub(crate) tmp_n: usize,
    pub(crate) cur_ret: Ty,
    pub(crate) cur_fn: Option<String>,
    /// Element type for the next empty-array literal (`let xs: int[] = [];`).
    pub(crate) hint_elem: Option<Ty>,
    /// Statements evaluating globals (run at the top of C `main`).
    pub(crate) ginit: String,
    pub(crate) ginit_indent: usize,
    pub(crate) protos: String,
    pub(crate) globals_decl: String,
}

impl Emitter {
    pub(crate) fn new(module: &Module) -> Result<Self, String> {
        if !module.imports.is_empty() {
            return Err("imports are evaluated by the interpreter".to_string());
        }
        let mut em = Emitter {
            struct_decls: BTreeMap::new(),
            structs: BTreeMap::new(),
            fns: BTreeMap::new(),
            fn_order: Vec::new(),
            globals: Vec::new(),
            global_tys: BTreeMap::new(),
            ret_tys: BTreeMap::new(),
            sigs: BTreeMap::new(),
            global_visible: usize::MAX,
            disp: BTreeSet::new(),
            dbg: BTreeSet::new(),
            insp: BTreeSet::new(),
            clones: BTreeSet::new(),
            eqs: BTreeSet::new(),
            lists: BTreeSet::new(),
            tuples: BTreeSet::new(),
            used_structs: BTreeSet::new(),
            resolving: BTreeSet::new(),
            out: String::new(),
            indent: 1,
            scopes: Vec::new(),
            bound: BTreeSet::new(),
            shadow_n: BTreeMap::new(),
            loop_stack: Vec::new(),
            label_n: 0,
            tmp_n: 0,
            cur_ret: Ty::Unit,
            cur_fn: None,
            hint_elem: None,
            ginit: String::new(),
            ginit_indent: 1,
            protos: String::new(),
            globals_decl: String::new(),
        };
        for decl in &module.declarations {
            match decl {
                Decl::Function(f) => {
                    let name = f.name.text.clone();
                    if em.fns.contains_key(&name) {
                        return Err(format!("duplicate function `{}`", name));
                    }
                    if f.body.is_none() {
                        return Err(format!("function `{}` has no body", name));
                    }
                    for p in &f.params {
                        if p.is_variadic || p.default.is_some() {
                            return Err(format!(
                                "`{}`: variadic/default parameters are interpreter-only",
                                name
                            ));
                        }
                    }
                    em.fn_order.push(name.clone());
                    em.fns.insert(name, f.clone());
                }
                Decl::Struct(s) => {
                    let name = s.name.text.clone();
                    if em.struct_decls.contains_key(&name) {
                        return Err(format!("duplicate struct `{}`", name));
                    }
                    let mut fields = Vec::new();
                    for fld in &s.fields {
                        if fields.iter().any(|(n, _): &(String, Type)| n == &fld.name.text) {
                            return Err(format!("duplicate field `{}` in `{}`", fld.name.text, name));
                        }
                        fields.push((fld.name.text.clone(), fld.ty.clone()));
                    }
                    em.struct_decls.insert(name, fields);
                }
                Decl::GlobalVar(g) => {
                    if em.globals.iter().any(|(n, _, _)| n == &g.name.text) {
                        return Err(format!("duplicate global `{}`", g.name.text));
                    }
                    em.globals
                        .push((g.name.text.clone(), g.ty.clone(), g.init.clone()));
                }
                // Classes/enums/traits/aliases/services are metadata for the
                // interpreter (no runtime effect); ignoring them matches its
                // behavior as long as no program construct references them —
                // and those references fail resolution below instead.
                Decl::Class(_)
                | Decl::Enum(_)
                | Decl::Trait(_)
                | Decl::Interface(_)
                | Decl::TypeAlias(_)
                | Decl::Effect(_)
                | Decl::Service(_) => {}
            }
        }
        if !em.fns.contains_key("main") {
            return Err("no `main` function".to_string());
        }
        for p in &em.fns["main"].params {
            if !p.is_variadic {
                return Err("`main` with parameters".to_string());
            }
        }
        if !em.fns["main"].params.is_empty() {
            return Err("`main` with parameters".to_string());
        }
        for name in &em.fn_order {
            let f = &em.fns[name];
            if !f.preconditions.is_empty() || !f.postconditions.is_empty() {
                return Err(format!("`{}`: contracts are interpreter-only", name));
            }
            if f.is_async {
                return Err(format!("`{}`: async is interpreter-only", name));
            }
            if !f.type_params.is_empty() {
                return Err(format!("`{}`: generics are interpreter-only", name));
            }
            if !f.where_clause.is_empty() {
                return Err(format!("`{}`: where clauses are interpreter-only", name));
            }
        }
        Ok(em)
    }

    // ---------------- type resolution ----------------

    pub(crate) fn ty_of(&mut self, t: &Type) -> Result<Ty, String> {
        match t {
            Type::Primitive(p) => Ok(match p {
                PrimitiveType::Bool => Ty::Bool,
                PrimitiveType::Char => Ty::Char,
                PrimitiveType::I8
                | PrimitiveType::I16
                | PrimitiveType::I32
                | PrimitiveType::I64
                | PrimitiveType::I128
                | PrimitiveType::U8
                | PrimitiveType::U16
                | PrimitiveType::U32
                | PrimitiveType::U64
                | PrimitiveType::U128 => Ty::Int,
                PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::Decimal => Ty::Float,
                PrimitiveType::Void => Ty::Unit,
                PrimitiveType::String => Ty::Str,
                PrimitiveType::Dynamic => return Err("dynamic type".to_string()),
            }),
            Type::Path(p) => {
                if p.segments.len() != 1 {
                    return Err(format!("type path `{}`", p.to_string()));
                }
                let name = &p.segments[0].text;
                match Self::prim_by_name(name) {
                    Some(ty) => Ok(ty),
                    None => {
                        self.resolve_struct(name)?;
                        Ok(Ty::Struct(name.clone()))
                    }
                }
            }
            Type::Slice(t) | Type::Array(t, _) => {
                let inner = self.ty_of(t)?;
                self.need_list(inner.clone());
                Ok(Ty::List(Box::new(inner)))
            }
            Type::Tuple(ts) => {
                let mut out = Vec::new();
                for t in ts {
                    out.push(self.ty_of(t)?);
                }
                self.need_tuple(out.clone());
                Ok(Ty::Tuple(out))
            }
            Type::Nullable(_) => Err("nullable types".to_string()),
            Type::Map(_, _) => Err("map types".to_string()),
            Type::Function(_, _) => Err("function types".to_string()),
            Type::Reference(_, _) => Err("reference types".to_string()),
            Type::Pointer(_, _) => Err("pointer types".to_string()),
            Type::Generic(_, _) => Err("generic types".to_string()),
            Type::Dynamic => Err("dynamic type".to_string()),
        }
    }

    fn prim_by_name(name: &str) -> Option<Ty> {
        Some(match name {
            "bool" => Ty::Bool,
            "char" => Ty::Char,
            "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "int" | "uint" => Ty::Int,
            "f32" | "f64" | "float" | "decimal" => Ty::Float,
            "void" => Ty::Unit,
            "string" | "String" | "str" => Ty::Str,
            _ => return None,
        })
    }

    pub(crate) fn resolve_struct(&mut self, name: &str) -> Result<(), String> {
        if self.structs.contains_key(name) {
            return Ok(());
        }
        if self.resolving.contains(name) {
            return Err(format!("recursive struct `{}`", name));
        }
        let raw = match self.struct_decls.get(name) {
            Some(r) => r.clone(),
            None => return Err(format!("unknown type `{}`", name)),
        };
        self.resolving.insert(name.to_string());
        let mut fields = Vec::new();
        for (fname, fty) in &raw {
            let ty = self.ty_of(fty)?;
            fields.push((fname.clone(), ty));
        }
        self.resolving.remove(name);
        self.used_structs.insert(name.to_string());
        self.structs
            .insert(name.to_string(), StructDef { name: name.to_string(), fields });
        Ok(())
    }

    // ---------------- worklist helpers ----------------

    pub(crate) fn need_disp(&mut self, t: &Ty) {
        if !matches!(t, Ty::Unknown | Ty::Panic) {
            self.disp.insert(t.clone());
        }
    }

    pub(crate) fn need_dbg(&mut self, t: &Ty) {
        if !matches!(t, Ty::Unknown | Ty::Panic) {
            self.dbg.insert(t.clone());
        }
    }

    pub(crate) fn need_insp(&mut self, t: &Ty) {
        if matches!(t, Ty::List(_) | Ty::Tuple(_) | Ty::Struct(_)) {
            self.insp.insert(t.clone());
        }
    }

    pub(crate) fn need_clone(&mut self, t: &Ty) {
        if t.needs_clone() {
            self.clones.insert(t.clone());
        }
    }

    pub(crate) fn need_eq(&mut self, a: &Ty, b: &Ty) {
        if matches!(a, Ty::Unknown | Ty::Panic) || matches!(b, Ty::Unknown | Ty::Panic) {
            return;
        }
        self.eqs.insert((a.clone(), b.clone()));
        if let Ty::List(x) = a {
            if let Ty::List(y) = b {
                self.need_eq(x, y);
            }
        }
        if let Ty::Tuple(xs) = a {
            if let Ty::Tuple(ys) = b {
                if xs.len() == ys.len() {
                    for (x, y) in xs.iter().zip(ys.iter()) {
                        self.need_eq(x, y);
                    }
                }
            }
        }
        if let Ty::Struct(x) = a {
            if let Ty::Struct(y) = b {
                if x == y {
                    let fields: Vec<Ty> = self
                        .structs
                        .get(x)
                        .map(|s| s.fields.iter().map(|(_, t)| t.clone()).collect())
                        .unwrap_or_default();
                    for f in fields {
                        self.need_eq(&f, &f);
                    }
                }
            }
        }
    }

    pub(crate) fn need_list(&mut self, elem: Ty) {
        if matches!(elem, Ty::Unknown) {
            return;
        }
        self.lists.insert(elem);
    }

    pub(crate) fn need_tuple(&mut self, ts: Vec<Ty>) {
        if ts.iter().any(|t| matches!(t, Ty::Unknown)) {
            return;
        }
        self.tuples.insert(ts);
    }

    // ---------------- output helpers ----------------

    pub(crate) fn line(&mut self, code: &str) {
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
        self.out.push_str(code);
        self.out.push('\n');
    }

    pub(crate) fn gline(&mut self, code: &str) {
        for _ in 0..self.ginit_indent {
            self.ginit.push_str("  ");
        }
        self.ginit.push_str(code);
        self.ginit.push('\n');
    }

    pub(crate) fn fresh_tmp(&mut self) -> String {
        let n = self.tmp_n;
        self.tmp_n += 1;
        format!("t{}", n)
    }

    pub(crate) fn fresh_label(&mut self) -> String {
        let n = self.label_n;
        self.label_n += 1;
        format!("L{}", n)
    }

    /// Declare `code` into a fresh temporary and return its text.
    pub(crate) fn tmpof(&mut self, ty: &Ty, code: &str) -> String {
        let name = self.fresh_tmp();
        let decl = format!("{} {} = {};", ty.ctype(), name, code);
        self.line(&decl);
        name
    }

    pub(crate) fn zero(&self, ty: &Ty) -> String {
        match ty {
            Ty::Int | Ty::Bool | Ty::Unit | Ty::Null | Ty::Panic | Ty::Unknown => "0LL".to_string(),
            Ty::Float => "0.0".to_string(),
            Ty::Char => "0U".to_string(),
            Ty::Str => "lx_s(\"\", 0)".to_string(),
            Ty::List(t) => format!("lx_l{}_new()", t.mangle()),
            Ty::Tuple(ts) => format!("({}){{0}}", tuple_cname(ts)),
            Ty::Struct(n) => format!("(LxS_{}){{0}}", esc(n)),
            Ty::Range => "(LxRange){0, 0, 0}".to_string(),
        }
    }

    /// Expression form of a runtime panic (never returns; typed so it can sit
    /// anywhere the interpreter would have errored).
    pub(crate) fn panic_expr(&mut self, msg: &str, ty: Ty) -> ExprC {
        let zero = self.zero(&ty);
        ExprC {
            text: format!("(lx_panic({}), {})", cstr_lit(msg), zero),
            ty,
            eff: true,
        }
    }

    /// A panic statement (no value needed).
    pub(crate) fn panic_stmt(&mut self, msg: &str) {
        let code = format!("lx_panic({});", cstr_lit(msg));
        self.line(&code);
    }

    // ---------------- module emission ----------------

    pub(crate) fn emit_module(&mut self) -> Result<(), String> {
        // Globals: resolve declared (or inferred) types, then emit storage.
        for (name, ty, init) in self.globals.clone() {
            let resolved = match &ty {
                Type::Primitive(PrimitiveType::Dynamic) => match &init {
                    Some(e) => {
                        self.cur_fn = None;
                        self.scopes.clear();
                        self.bound.clear();
                        self.peek_expr(e)
                    }
                    None => return Err(format!("global `{}` needs a type", name)),
                },
                t => self.ty_of(t)?,
            };
            if resolved == Ty::Unknown {
                return Err(format!("cannot infer type of global `{}`", name));
            }
            self.global_tys.insert(name.clone(), resolved.clone());
            self.globals_decl
                .push_str(&format!("static {} g_{};\n", resolved.ctype(), esc(&name)));
        }

        // Signatures and prototypes.
        for name in self.fn_order.clone() {
            let f = self.fns[&name].clone();
            let ret = self.ret_tys[&name].clone();
            let mut params = Vec::new();
            for p in &f.params {
                let ty = self.ty_of(&p.ty)?;
                params.push((format!("v_{}", esc(&p.name.text)), ty));
            }
            let cname = format!("fx_{}", esc(&name));
            let plist = if params.is_empty() {
                "void".to_string()
            } else {
                params
                    .iter()
                    .map(|(n, t)| format!("{} {}", t.ctype(), n))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            self.protos
                .push_str(&format!("static {} {}({});\n", ret.ctype(), cname, plist));
            self.sigs
                .insert(name, Sig { cname, ret, params });
        }

        // Global initializers (evaluated in order at the top of `main`).
        for (i, (name, _, init)) in self.globals.clone().into_iter().enumerate() {
            if let Some(init) = &init {
                self.cur_fn = None;
                self.global_visible = i;
                self.scopes.clear();
                self.bound.clear();
                let v = self.emit_expr(&init)?;
                let ty = self.global_tys[&name].clone();
                let st = self.need(&v, &format!("initializer of global `{}`", name))?;
                self.gline(&format!("g_{} = {};", esc(&name), st.text));
            }
        }

        // Function bodies.
        for name in self.fn_order.clone() {
            let f = self.fns[&name].clone();
            self.emit_fn(&name, &f)?;
        }
        Ok(())
    }

    fn emit_fn(&mut self, name: &str, f: &Function) -> Result<(), String> {
        self.scopes.clear();
        self.bound.clear();
        self.shadow_n.clear();
        self.loop_stack.clear();
        self.tmp_n = 0;
        self.cur_fn = Some(name.to_string());
        self.cur_ret = self.ret_tys[name].clone();
        if self.cur_ret == Ty::Unknown {
            return Err(format!("cannot infer return type of `{}`", name));
        }
        self.out.push_str(&format!(
            "static {} {}({}) {{\n",
            self.cur_ret.ctype(),
            self.sigs[name].cname,
            self.sigs[name]
                .params
                .iter()
                .map(|(n, t)| format!("{} {}", t.ctype(), n))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        self.indent = 1;
        self.line("lx_depth += 1;");
        self.line("if (lx_depth > 500) { lx_depth -= 1; lx_panic(\"call stack overflow (possible infinite recursion)\"); }");
        if self.cur_ret != Ty::Unit {
            let rv = self.cur_ret.ctype();
            let z = self.zero(&self.cur_ret.clone());
            self.line(&format!("{} lx_rv = {};", rv, z));
        }
        let mut scope = HashMap::new();
        for p in &f.params {
            let ty = self.ty_of(&p.ty)?;
            let cn = format!("v_{}", esc(&p.name.text));
            scope.insert(p.name.text.clone(), (cn, ty));
            self.bound.insert(p.name.text.clone());
        }
        self.scopes.push(scope);
        self.emit_block(&f.body.clone().unwrap())?;
        self.scopes.pop();
        if self.cur_ret == Ty::Unit {
            self.line("lx_depth -= 1;");
            self.line("return 0;");
        } else {
            self.line("lx_depth -= 1;");
            self.line("return lx_rv;");
        }
        self.out.push_str("}\n");
        Ok(())
    }

    // ---------------- binding / scoping ----------------

    /// Bind `name` in the innermost scope, giving it a unique C variable.
    pub(crate) fn bind(&mut self, name: &str, ty: Ty) -> Result<String, String> {
        let cbase = format!("v_{}", esc(name));
        let cname = {
            let cnt = self.shadow_n.entry(name.to_string()).or_insert(0);
            let c = if *cnt == 0 {
                cbase.clone()
            } else {
                format!("{}_{}", cbase, cnt + 1)
            };
            *cnt += 1;
            let cty = ty.ctype();
            let z = self.zero(&ty);
            self.line(&format!("{} {} = {};", cty, c, z));
            c
        };
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.to_string(), (cname.clone(), ty));
        }
        self.bound.insert(name.to_string());
        Ok(cname)
    }

    /// Declare + initialize a new binding (used by `let`, patterns, loops).
    pub(crate) fn bind_init(&mut self, name: &str, ty: Ty, init: &str) -> Result<(), String> {
        let cbase = format!("v_{}", esc(name));
        let cname = {
            let cnt = self.shadow_n.entry(name.to_string()).or_insert(0);
            let c = if *cnt == 0 {
                cbase.clone()
            } else {
                format!("{}_{}", cbase, cnt + 1)
            };
            *cnt += 1;
            let cty = ty.ctype();
            self.line(&format!("{} {} = {};", cty, c, init));
            c
        };
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.to_string(), (cname, ty));
        }
        self.bound.insert(name.to_string());
        Ok(())
    }

    /// Look up a visible binding's C variable + type.
    pub(crate) fn lookup(&self, name: &str) -> Option<(String, Ty)> {
        for scope in self.scopes.iter().rev() {
            if let Some((c, t)) = scope.get(name) {
                return Some((c.clone(), t.clone()));
            }
        }
        None
    }

    /// Resolve a name for reading. `Err` → fall back to the interpreter
    /// (which then produces the exact runtime behavior).
    pub(crate) fn read_name(&mut self, name: &str) -> Result<Option<(String, Ty)>, String> {
        if let Some(hit) = self.lookup(name) {
            return Ok(Some(hit));
        }
        if self.bound.contains(name) {
            return Err(format!("`{}` may be read before it is bound", name));
        }
        if let Some(ty) = self.global_tys.get(name) {
            if self.cur_fn.is_none() {
                // In a global initializer only already-declared globals are
                // visible to the interpreter; a later global reads as
                // `undefined variable` there, so emit the same panic.
                let idx = self.globals.iter().position(|(n, _, _)| n == name);
                if idx.map(|i| i >= self.global_visible).unwrap_or(true) {
                    return Ok(None);
                }
            }
            return Ok(Some((format!("g_{}", esc(name)), ty.clone())));
        }
        Ok(None)
    }

    pub(crate) fn need<'a>(&self, v: &'a ExprC, what: &str) -> Result<&'a ExprC, String> {
        if v.ty == Ty::Unknown {
            return Err(format!("unsupported value in {}", what));
        }
        Ok(v)
    }
}

impl Emitter {
    fn finish(&mut self, types: &str, protos: &str, bodies: &str) -> String {
        let mut s = String::new();
        s.push_str(RUNTIME);
        s.push_str(types);
        s.push_str(protos);
        s.push_str(bodies);
        s.push_str(&self.protos);
        s.push_str(&self.globals_decl);
        s.push_str(&self.out);
        s.push_str("\nint main(void) {\n");
        s.push_str("  lx_depth = 0;\n");
        s.push_str(&self.ginit);
        s.push_str("  fx_main();\n");
        s.push_str("  lx_ofinish();\n");
        s.push_str("  return 0;\n}\n");
        s.push_str(rt::RUNTIME_TAIL);
        s
    }
}
