//! Tree-walking interpreter for `lex run`.
//!
//! The legacy `lex run` path statically scans print call-sites against a
//! last-wins global variable map, so `println(o.name)`, `println(total)` or
//! `println(soma(1, 2))` print the *source text* instead of the value, every
//! branch of `if/else`/`switch`/`match` prints, and loops print once with the
//! wrong bindings. This module executes the parsed [`Module`] for real:
//! statements run in order from `main`, with scoped bindings, loops,
//! conditions, calls (with argument binding), struct literals/fields, string
//! interpolation, casts and ranges.
//!
//! Anything outside the supported subset returns [`InterpError::Unsupported`]
//! and the caller falls back to the legacy mock output, so programs using
//! exotic constructs behave exactly as before instead of erroring.

use lexicon_parser::{
    BinOp, Block, Decl, Expr, Function, InterpolatedPart, Literal, MatchArm, Module, Pattern,
    Stmt, Struct, UnOp, Visibility,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// Budgets: `lex run` must never hang the terminal on `loop {}` or deep
// recursion — these surface as clear runtime errors instead. 50M steps is
// enough for 1M-iteration benches (~15M steps) while still catching real
// infinite loops within a couple of seconds.
const STEP_BUDGET: u64 = 50_000_000;
const MAX_DEPTH: usize = 500;
const MAX_OUTPUT_CHARS: usize = 1_000_000;

#[derive(Debug, Clone)]
pub enum Val {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Null,
    Unit,
    List(Vec<Val>),
    Tuple(Vec<Val>),
    Range { start: i64, end: i64, inclusive: bool },
    Struct { name: String, fields: Vec<(String, Val)> },
    /// First-class function value (lambda `|x| x + 1`).
    /// `captured` is a flattened snapshot of the visible bindings at
    /// creation time, so comparators/predicates keep working when they
    /// are passed into another module (e.g. `slices::Map(xs, |x| x)`).
    Func(FuncVal),
}

#[derive(Debug, Clone)]
pub struct FuncVal {
    pub params: Vec<String>,
    pub body: Box<lexicon_parser::Expr>,
    pub captured: std::collections::HashMap<String, Val>,
}

#[derive(Debug, Clone)]
pub enum InterpError {
    /// Construct outside the supported subset — caller falls back to mock.
    Unsupported(String),
    /// Real execution failure (undefined variable, failed assert, ...).
    Runtime(String),
}

impl std::fmt::Display for InterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpError::Unsupported(m) => write!(f, "unsupported construct: {}", m),
            InterpError::Runtime(m) => write!(f, "{}", m),
        }
    }
}

type IResult<T> = Result<T, InterpError>;

fn unsupported<T>(msg: impl Into<String>) -> IResult<T> {
    Err(InterpError::Unsupported(msg.into()))
}

fn runtime<T>(msg: impl Into<String>) -> IResult<T> {
    Err(InterpError::Runtime(msg.into()))
}

#[derive(Debug, Clone)]
enum Flow {
    Next,
    Break,
    Continue,
    Return(Val),
}

impl Val {
    /// User-facing rendering used by `println` (strings print raw).
    fn display(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Float(x) => {
                // `42.0` renders as `42` (matches the mock's integer look).
                if x.fract() == 0.0 && x.is_finite() {
                    format!("{}", *x as i64)
                } else {
                    format!("{}", x)
                }
            }
            Val::Bool(b) => b.to_string(),
            Val::Str(s) => s.clone(),
            Val::Char(c) => c.to_string(),
            Val::Null => "null".to_string(),
            Val::Unit => String::new(),
            Val::List(items) => format!(
                "[{}]",
                items.iter().map(|v| v.debug_elem()).collect::<Vec<_>>().join(", ")
            ),
            Val::Tuple(items) => format!(
                "({})",
                items.iter().map(|v| v.debug_elem()).collect::<Vec<_>>().join(", ")
            ),
            Val::Range { start, end, inclusive } => {
                if *inclusive {
                    format!("{}..={}", start, end)
                } else {
                    format!("{}..{}", start, end)
                }
            }
            Val::Struct { name, fields } => format!(
                "{} {{ {} }}",
                name,
                fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.debug_elem()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Val::Func(_) => "<fn>".to_string(),
        }
    }

    /// Rendering nested inside containers (strings stay quoted).
    fn debug_elem(&self) -> String {
        match self {
            Val::Str(s) => format!("{:?}", s),
            Val::Char(c) => format!("{:?}", c),
            other => other.display(),
        }
    }

    /// `inspect(v)` rendering — everything debug-style.
    fn inspect(&self) -> String {
        match self {
            Val::Str(s) => format!("{:?}", s),
            Val::Char(c) => format!("{:?}", c),
            Val::List(items) => format!(
                "[{}]",
                items.iter().map(|v| v.inspect()).collect::<Vec<_>>().join(", ")
            ),
            Val::Tuple(items) => format!(
                "({})",
                items.iter().map(|v| v.inspect()).collect::<Vec<_>>().join(", ")
            ),
            Val::Struct { name, fields } => format!(
                "{} {{ {} }}",
                name,
                fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.inspect()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Val::Func(_) => "<fn>".to_string(),
            other => other.display(),
        }
    }

    fn to_json(&self) -> serde_json::Value {
        match self {
            Val::Int(n) => serde_json::Value::from(*n),
            Val::Float(x) => serde_json::json!(*x),
            Val::Bool(b) => serde_json::Value::from(*b),
            Val::Str(s) => serde_json::Value::from(s.clone()),
            Val::Char(c) => serde_json::Value::from(c.to_string()),
            Val::Null | Val::Unit => serde_json::Value::Null,
            Val::List(items) => items.iter().map(|v| v.to_json()).collect(),
            Val::Tuple(items) => items.iter().map(|v| v.to_json()).collect(),
            Val::Range { start, end, inclusive } => {
                let mut v = Vec::new();
                if *inclusive {
                    for i in *start..=*end {
                        v.push(serde_json::Value::from(i));
                    }
                } else {
                    for i in *start..*end {
                        v.push(serde_json::Value::from(i));
                    }
                }
                serde_json::Value::Array(v)
            }
            Val::Struct { fields, .. } => {
                let mut m = serde_json::Map::new();
                for (k, v) in fields {
                    m.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(m)
            }
            // Functions are opaque to JSON (same policy as before for
            // exotic values): encode as null instead of failing the run.
            Val::Func(_) => serde_json::Value::Null,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Val::Int(_) => "int",
            Val::Float(_) => "float",
            Val::Bool(_) => "bool",
            Val::Str(_) => "String",
            Val::Char(_) => "char",
            Val::Null => "null",
            Val::Unit => "void",
            Val::List(_) => "List",
            Val::Tuple(_) => "Tuple",
            Val::Range { .. } => "Range",
            Val::Struct { .. } => "struct",
            Val::Func(_) => "func",
        }
    }

    /// User-facing runtime type name for `typeOf(x)` (JS-style
    /// introspection): precise primitive names, and the DECLARED name for
    /// user structs (`User`, `Error`, … — Go `%T`-like) instead of the
    /// generic `struct`.
    fn type_of(&self) -> String {
        match self {
            Val::Struct { name, .. } => name.clone(),
            other => other.type_name().to_string(),
        }
    }

    fn equals(&self, other: &Val) -> bool {
        match (self, other) {
            (Val::Int(a), Val::Int(b)) => a == b,
            (Val::Float(a), Val::Float(b)) => a == b,
            (Val::Int(a), Val::Float(b)) => (*a as f64) == *b,
            (Val::Float(a), Val::Int(b)) => *a == (*b as f64),
            (Val::Bool(a), Val::Bool(b)) => a == b,
            (Val::Str(a), Val::Str(b)) => a == b,
            (Val::Char(a), Val::Char(b)) => a == b,
            (Val::Null, Val::Null) => true,
            (Val::Unit, Val::Unit) => true,
            (Val::List(a), Val::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            (Val::Tuple(a), Val::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            (
                Val::Struct { name: na, fields: fa },
                Val::Struct { name: nb, fields: fb },
            ) => {
                na == nb
                    && fa.len() == fb.len()
                    && fa.iter().zip(fb.iter()).all(|((ka, va), (kb, vb))| ka == kb && va.equals(vb))
            }
            // Function identity is deliberately opaque: two lambdas are
            // never `==`, even when their bodies match.
            (Val::Func(_), _) | (_, Val::Func(_)) => false,
            _ => false,
        }
    }
}

fn json_to_val(v: &serde_json::Value) -> Val {
    match v {
        serde_json::Value::Null => Val::Null,
        serde_json::Value::Bool(b) => Val::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Val::Int(i)
            } else if let Some(f) = n.as_f64() {
                Val::Float(f)
            } else {
                Val::Str(n.to_string())
            }
        }
        serde_json::Value::String(s) => Val::Str(s.clone()),
        serde_json::Value::Array(items) => Val::List(items.iter().map(json_to_val).collect()),
        serde_json::Value::Object(map) => Val::Struct {
            name: "Object".to_string(),
            fields: map.iter().map(|(k, v)| (k.clone(), json_to_val(v))).collect(),
        },
    }
}

/// A Lex module loaded from a `.lex` file on disk (`import std::strings;`).
/// Functions/structs/globals are also flattened into the interpreter tables
/// so code in the entry file can call imported names unqualified; qualified
/// `mod::name` calls always resolve exactly through this record. While a
/// module's own function runs, unqualified lookups are resolved against this
/// record *first* (`Cx::cur_mod`), so two packages that both define `Sum`
/// or `words` cannot shadow each other from inside their own bodies.
#[derive(Debug, Default)]
struct LoadedMod {
    funcs: HashMap<String, std::rc::Rc<Function>>,
    structs: HashMap<String, std::rc::Rc<Struct>>,
    globals: HashMap<String, Val>,
    /// Names declared `pub` — the only ones callable from importing code.
    public: HashSet<String>,
}

struct Cx {
    funcs: HashMap<String, std::rc::Rc<Function>>,
    structs: HashMap<String, std::rc::Rc<Struct>>,
    globals: HashMap<String, Val>,
    scopes: Vec<HashMap<String, Val>>,
    output: String,
    steps: u64,
    depth: usize,
    mods: HashMap<String, LoadedMod>,
    /// Module whose function is currently executing (None = entry file).
    cur_mod: Option<String>,
}

impl Cx {
    fn step(&mut self) -> IResult<()> {
        self.steps += 1;
        if self.steps > STEP_BUDGET {
            return runtime("execution limit exceeded (possible infinite loop)");
        }
        Ok(())
    }

    fn push_output(&mut self, s: &str) -> IResult<()> {
        self.step()?;
        self.output.push_str(s);
        if self.output.len() > MAX_OUTPUT_CHARS {
            return runtime("output limit exceeded (possible runaway print loop)");
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<Val> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        if let Some(m) = &self.cur_mod {
            if let Some(v) = self.mods.get(m).and_then(|m| m.globals.get(name)) {
                return Some(v.clone());
            }
        }
        self.globals.get(name).cloned()
    }

    /// Unqualified function lookup: the executing module's own definition
    /// wins, then the flattened table. Returns the function and the module
    /// it came from (None = entry file), which becomes the callee's module
    /// context.
    fn user_fn(&self, name: &str) -> Option<(std::rc::Rc<Function>, Option<String>)> {
        if let Some(m) = &self.cur_mod {
            if let Some(f) = self.mods.get(m).and_then(|m| m.funcs.get(name)) {
                return Some((f.clone(), Some(m.clone())));
            }
        }
        self.funcs.get(name).cloned().map(|f| (f, None))
    }

    /// Unqualified struct lookup: same module-first rule as [`Cx::user_fn`].
    fn user_struct(&self, name: &str) -> Option<std::rc::Rc<Struct>> {
        if let Some(m) = &self.cur_mod {
            if let Some(s) = self.mods.get(m).and_then(|m| m.structs.get(name)) {
                return Some(s.clone());
            }
        }
        self.structs.get(name).cloned()
    }

    /// Assign to an existing binding (innermost first), else create locally.
    fn assign(&mut self, name: &str, val: Val) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return;
            }
        }
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), val);
            return;
        }
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.to_string(), val);
        }
    }

    fn truthy(&self, v: &Val) -> IResult<bool> {
        match v {
            Val::Bool(b) => Ok(*b),
            other => runtime(format!(
                "condition must be `bool`, found {}",
                other.type_name()
            )),
        }
    }
}

/// Runs `f` on a thread with a large stack (256 MiB). Both the parser
/// (nested expressions) and this tree-walking interpreter (deep call
/// chains) recurse natively; the default main-thread stack (1 MiB on
/// Windows) would abort the whole process with a stack overflow instead
/// of a clean diagnostic. Depth budgets (`MAX_DEPTH`,
/// `MAX_PARSE_DEPTH`) then always fire first. `pub(crate)` so the
/// compiler's own parse sites share the protection.
pub(crate) fn run_big_stack<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match std::thread::Builder::new()
        .name("lex-run".to_string())
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
    {
        Ok(handle) => handle.join().map_err(|_| "execution thread panicked".to_string()),
        Err(e) => Err(format!("cannot spawn execution thread: {}", e)),
    }
}

/// Entry point: execute `main` with no arguments, collecting printed output.
pub fn run_module(module: &Module) -> Result<String, InterpError> {
    let owned = module.clone();
    match run_big_stack(move || run_module_inner(&owned)) {
        Ok(r) => r,
        Err(msg) => Err(InterpError::Runtime(msg)),
    }
}

fn run_module_inner(module: &Module) -> Result<String, InterpError> {
    let mut cx = setup_cx(module)?;
    let main = cx
        .funcs
        .get("main")
        .cloned()
        .ok_or_else(|| InterpError::Unsupported("no `main` function".to_string()))?;
    if !main.params.is_empty() {
        return unsupported("`main` with parameters");
    }
    if main.body.is_none() {
        return unsupported("`main` without a body");
    }
    call_user(&mut cx, &main, Vec::new())?;
    // Janelas gráficas abertas: mantém o processo vivo até o usuário fechar
    // (apps orientados a eventos). CI/testes usam LEXICON_GUI_AUTOQUIT.
    #[cfg(feature = "gui")]
    crate::gfx::wait_windows();
    Ok(cx.output)
}

/// Shared setup: fresh scope tables + resolved imports + registered
/// declarations + evaluated globals. Used by [`run_module`] and
/// [`run_tests`] alike so tests execute in the exact same environment
/// as `lex run`.
fn setup_cx(module: &Module) -> Result<Cx, InterpError> {
    let mut cx = Cx {
        funcs: HashMap::new(),
        structs: HashMap::new(),
        globals: HashMap::new(),
        scopes: Vec::new(),
        output: String::new(),
        steps: 0,
        depth: 0,
        mods: HashMap::new(),
        cur_mod: None,
    };
    // Phase 0 (Go-parity plan §Fase 0): resolve `import a::b` to `a/b.lex`
    // under the SDK roots, parse, and register the module's declarations.
    // Imports load before the main module's own decls so initializers on
    // either side can call imported functions.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    load_imports(&mut cx, module, &cwd, &mut visited, &mut stack)?;
    for decl in &module.declarations {
        match decl {
            Decl::Function(f) => {
                cx.funcs.insert(f.name.text.clone(), std::rc::Rc::new(f.clone()));
            }
            Decl::Struct(s) => {
                cx.structs.insert(s.name.text.clone(), std::rc::Rc::new(s.clone()));
            }
            Decl::GlobalVar(g) => {
                let v = match &g.init {
                    Some(init) => cx.eval(init)?,
                    None => Val::Null,
                };
                cx.globals.insert(g.name.text.clone(), v);
            }
            // Types, traits, classes, services, aliases: no runtime effect.
            _ => {}
        }
    }
    Ok(cx)
}

/// One executed `@Test` case.
#[derive(Debug, Clone)]
pub struct TestOutcome {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Entry point: discover every `pub fn` carrying `@Test` and execute it
/// with no arguments — the real Jest-like run (not a mock count).
/// Convention: a test PASSES when it returns truthy or `void`/`Unit`;
/// `false` or a runtime abort FAILs with the message. Tests needing
/// arguments receive `null`s (documented: write zero-arg tests).
pub fn run_tests(module: &Module) -> Result<Vec<TestOutcome>, InterpError> {
    let owned = module.clone();
    match run_big_stack(move || run_tests_inner(&owned)) {
        Ok(r) => r,
        Err(msg) => Err(InterpError::Runtime(msg)),
    }
}

fn run_tests_inner(module: &Module) -> Result<Vec<TestOutcome>, InterpError> {
    let mut cx = setup_cx(module)?;
    // Declaration order (stable, readable output).
    let mut names: Vec<String> = Vec::new();
    for decl in &module.declarations {
        if let Decl::Function(f) = decl {
            if f.attrs.iter().any(|a| a.name.text == "Test") {
                names.push(f.name.text.clone());
            }
        }
    }
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let func = cx.funcs.get(&name).cloned().ok_or_else(|| {
            InterpError::Runtime(format!("test `{}` disappeared", name))
        })?;
        match call_user(&mut cx, &func, Vec::new()) {
            Ok(Val::Bool(false)) => out.push(TestOutcome {
                name,
                passed: false,
                detail: "returned false".to_string(),
            }),
            Ok(_) => out.push(TestOutcome {
                name,
                passed: true,
                detail: String::new(),
            }),
            Err(InterpError::Runtime(msg)) => out.push(TestOutcome {
                name,
                passed: false,
                detail: msg,
            }),
            Err(InterpError::Unsupported(msg)) => out.push(TestOutcome {
                name,
                passed: false,
                detail: format!("unsupported construct: {}", msg),
            }),
        }
    }
    Ok(out)
}

/// Builtin modules provided by the interpreter itself (`Console`, `Json`,
/// `Env`, `Http`, ...). An unresolvable import whose last segment names one
/// of these is a legacy decorative import (`import std::console;`) — kept as
/// a no-op for backward compatibility instead of a hard error.
fn is_builtin_module(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "console" | "http" | "json" | "env" | "list" | "grpc" | "wasm" | "webview" | "crypto"
            | "file" | "db" | "window" | "menu"
            | "process" | "atomic" | "math" | "text" | "rand" | "hash"
            | "time" | "sys"
            // Motor gráfico nativo (`gfx.rs`): imports decorativos não quebram.
            | "canvas" | "input" | "gpu" | "label" | "button"
            | "textfield" | "checkbox" | "menubar" | "menuitem"
    )
}

/// Directories searched for `import a::b::c` → `a/b/c.lex`, in order:
/// the importing file's directory, then the local trees, then the running
/// binary's SDK layout, then the per-user installed SDK
/// (`~/.lexicon/sdk`, managed by first-run auto-install), then every
/// `$LEX_PATH` entry.
///
/// `pub(crate)`: the language server resolves go-to-definition through the
/// exact same roots, so Ctrl+Click lands where the code really runs from.
pub(crate) fn module_search_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from(".").join("lib"),
        PathBuf::from(".").join("build").join("sdk").join("lib"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("lib"));
            roots.push(dir.join("..").join("lib"));
        }
    }
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        if !home.is_empty() {
            roots.push(PathBuf::from(home).join(".lexicon").join("sdk").join("lib"));
        }
    }
    if let Ok(lex_path) = std::env::var("LEX_PATH") {
        // Platform-correct split (`;` on Windows, `:` elsewhere — a raw
        // `split(':')` would break `C:\…` roots on Windows, a raw
        // `split(';')` would break exotic Unix names).
        for part in std::env::split_paths(&lex_path) {
            if !part.as_os_str().is_empty() {
                roots.push(part);
            }
        }
    }
    roots
}

/// First hit wins, mirroring the interpreter's own resolution order.
/// `pub(crate)` for go-to-definition (see [`module_search_roots`]).
pub(crate) fn find_module_file(segs: &[String], base_dir: &Path) -> Option<PathBuf> {
    let mut rel = PathBuf::new();
    for s in segs {
        rel.push(s);
    }
    rel.set_extension("lex");
    let local = base_dir.join(&rel);
    if local.is_file() {
        return Some(local);
    }
    // Pacotes instalados pelo gerenciador (`lex mod add`/`install`):
    // `lex_packages/` na raiz do projeto (walk-up a partir do importador).
    // Prioridade sobre a SDK — dependências do projeto sombreiam a stdlib.
    let mut dir = base_dir.to_path_buf();
    for _ in 0..8 {
        let pkgs = dir.join("lex_packages");
        if pkgs.is_dir() {
            let cand = pkgs.join(&rel);
            if cand.is_file() {
                return Some(cand);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    for root in module_search_roots() {
        let cand = root.join(&rel);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

fn parse_module_file(src: &str, label: &str) -> Result<Module, InterpError> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize();
    let mut parser = Parser::with_file(tokens, label);
    let module = parser
        .parse()
        .map_err(|e| InterpError::Runtime(format!("{}: {}", label, e)))?;
    let errors = parser.take_errors();
    if !errors.is_empty() {
        let msg = errors
            .iter()
            .map(|e| format!("{}:{}:{}: {}", label, e.line, e.column, e.message))
            .collect::<Vec<_>>()
            .join("\n");
        return runtime(msg);
    }
    Ok(module)
}

/// Load every import of `module` (recursively, depth-first). Cycles are a
/// hard error; diamond imports load once via `visited`.
fn load_imports(
    cx: &mut Cx,
    module: &Module,
    base_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    stack: &mut Vec<String>,
) -> IResult<()> {
    for imp in &module.imports {
        let segs: Vec<String> = imp.path.segments.iter().map(|s| s.text.clone()).collect();
        let key = imp
            .alias
            .as_ref()
            .map(|a| a.text.clone())
            .unwrap_or_else(|| segs.last().cloned().unwrap_or_default());
        if cx.mods.contains_key(&key) {
            continue;
        }
        let file = match find_module_file(&segs, base_dir) {
            Some(f) => f,
            None => {
                // `import std::console;` and friends name builtin modules that
                // have no on-disk source — the Rust implementations in
                // `call_module` still back the calls, so the import is a
                // no-op instead of an error. Anything else is a real miss.
                let last = segs.last().map(|s| s.to_ascii_lowercase());
                if last.as_deref().map(is_builtin_module).unwrap_or(false) {
                    continue;
                }
                return Err(InterpError::Runtime(format!(
                    "cannot find module `{}` (tried ./lib, ./build/sdk/lib, exe/lib, $LEX_PATH)",
                    imp.path.to_string()
                )));
            }
        };
        let canon = std::fs::canonicalize(&file).unwrap_or_else(|_| file.clone());
        if visited.contains(&canon) {
            continue;
        }
        let canon_str = canon.to_string_lossy().to_string();
        if stack.iter().any(|s| s == &canon_str) {
            let mut chain = stack.clone();
            chain.push(imp.path.to_string());
            return runtime(format!("import cycle: {}", chain.join(" -> ")));
        }
        stack.push(canon_str);
        let src = std::fs::read_to_string(&file)
            .map_err(|e| InterpError::Runtime(format!("cannot read {}: {}", file.display(), e)))?;
        let sub = parse_module_file(&src, &file.to_string_lossy())?;
        let sub_dir = file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| base_dir.to_path_buf());
        load_imports(cx, &sub, &sub_dir, visited, stack)?;

        // Pass 1: register functions/structs (flattened + namespaced) so
        // global initializers can call them.
        let mut loaded = LoadedMod::default();
        for decl in &sub.declarations {
            match decl {
                Decl::Function(f) => {
                    if matches!(f.visibility, Visibility::Public) {
                        loaded.public.insert(f.name.text.clone());
                    }
                    loaded.funcs.insert(f.name.text.clone(), std::rc::Rc::new(f.clone()));
                    cx.funcs.entry(f.name.text.clone()).or_insert_with(|| std::rc::Rc::new(f.clone()));
                }
                Decl::Struct(s) => {
                    if matches!(s.visibility, Visibility::Public) {
                        loaded.public.insert(s.name.text.clone());
                    }
                    loaded.structs.insert(s.name.text.clone(), std::rc::Rc::new(s.clone()));
                    cx.structs.entry(s.name.text.clone()).or_insert_with(|| std::rc::Rc::new(s.clone()));
                }
                _ => {}
            }
        }
        cx.mods.insert(key.clone(), loaded);
        // Pass 2: evaluate module-level globals.
        for decl in &sub.declarations {
            if let Decl::GlobalVar(g) = decl {
                if matches!(g.visibility, Visibility::Public) {
                    if let Some(m) = cx.mods.get_mut(&key) {
                        m.public.insert(g.name.text.clone());
                    }
                }
                let v = match &g.init {
                    Some(init) => cx.eval(init)?,
                    None => Val::Null,
                };
                if let Some(m) = cx.mods.get_mut(&key) {
                    m.globals.insert(g.name.text.clone(), v.clone());
                }
                cx.globals.entry(g.name.text.clone()).or_insert(v);
            }
        }
        stack.pop();
        visited.insert(canon);
    }
    Ok(())
}

/// Calls `func` with `home` installed as the module context, restoring the
/// caller's context afterwards. Use this for every user-function dispatch so
/// unqualified lookups inside the callee see the module that defines it.
fn call_user_home(
    cx: &mut Cx,
    func: &Function,
    args: Vec<Val>,
    home: Option<String>,
) -> IResult<Val> {
    let outer = std::mem::replace(&mut cx.cur_mod, home);
    let r = call_user(cx, func, args);
    cx.cur_mod = outer;
    r
}

fn call_user(cx: &mut Cx, func: &Function, args: Vec<Val>) -> IResult<Val> {
    cx.depth += 1;
    if cx.depth > MAX_DEPTH {
        cx.depth -= 1;
        return runtime("call stack overflow (possible infinite recursion)");
    }
    let body = func
        .body
        .clone()
        .ok_or_else(|| InterpError::Unsupported(format!("`{}` without a body", func.name.text)))?;
    let mut scope = HashMap::new();
    for (param, val) in func.params.iter().zip(args.into_iter()) {
        scope.insert(param.name.text.clone(), val);
    }
    // Missing (defaulted/variadic) params evaluate their defaults now.
    for param in func.params.iter().skip(scope.len()) {
        let v = match &param.default {
            Some(d) => cx.eval(d)?,
            None => Val::Null,
        };
        scope.insert(param.name.text.clone(), v);
    }
    cx.scopes.push(scope);
    // `defer` blocks run LIFO when the call finishes (any exit path).
    let mut defers: Vec<Block> = Vec::new();
    let flow = exec_block(cx, &body, &mut defers);
    for block in defers.iter().rev() {
        cx.scopes.push(HashMap::new());
        let _ = exec_block(cx, block, &mut Vec::new());
        cx.scopes.pop();
    }
    cx.scopes.pop();
    cx.depth -= 1;
    match flow {
        Ok(Flow::Next) => Ok(Val::Unit),
        Ok(Flow::Return(v)) => Ok(v),
        Ok(Flow::Break) => runtime("`break` outside of a loop"),
        Ok(Flow::Continue) => runtime("`continue` outside of a loop"),
        Err(e) => Err(e),
    }
}

/// Invoke a first-class lambda value with already-evaluated arguments.
fn call_lambda(cx: &mut Cx, func: &FuncVal, args: Vec<Val>) -> IResult<Val> {
    if args.len() != func.params.len() {
        return runtime(format!(
            "lambda takes {} argument(s), got {}",
            func.params.len(),
            args.len()
        ));
    }
    cx.depth += 1;
    if cx.depth > MAX_DEPTH {
        cx.depth -= 1;
        return runtime("call stack overflow (possible infinite recursion)");
    }
    let mut scope = func.captured.clone();
    for (name, val) in func.params.iter().zip(args.into_iter()) {
        scope.insert(name.clone(), val);
    }
    cx.scopes.push(scope);
    let result = cx.eval(&func.body);
    cx.scopes.pop();
    cx.depth -= 1;
    result
}

fn exec_block(cx: &mut Cx, block: &Block, defers: &mut Vec<Block>) -> IResult<Flow> {
    cx.scopes.push(HashMap::new());
    let mut flow = Flow::Next;
    for stmt in &block.statements {
        flow = exec_stmt(cx, stmt, defers)?;
        match flow {
            Flow::Next => {}
            _ => break,
        }
    }
    cx.scopes.pop();
    Ok(flow)
}

fn bind_pattern(
    pattern: &Pattern,
    val: Val,
    scope: &mut HashMap<String, Val>,
) -> IResult<()> {
    match pattern {
        Pattern::Ident(id) => {
            scope.insert(id.text.clone(), val);
            Ok(())
        }
        Pattern::Wildcard(_) => Ok(()),
        Pattern::Tuple(parts) => match val {
            Val::Tuple(items) | Val::List(items) => {
                if items.len() != parts.len() {
                    return runtime(format!(
                        "destructuring arity mismatch: {} pattern(s), {} value(s)",
                        parts.len(),
                        items.len()
                    ));
                }
                for (p, v) in parts.iter().zip(items.into_iter()) {
                    bind_pattern(p, v, scope)?;
                }
                Ok(())
            }
            other => runtime(format!(
                "cannot destructure {} as a tuple",
                other.type_name()
            )),
        },
        _ => unsupported("complex destructuring pattern in `let`"),
    }
}

fn exec_stmt(cx: &mut Cx, stmt: &Stmt, defers: &mut Vec<Block>) -> IResult<Flow> {
    cx.step()?;
    match stmt {
        Stmt::Expr(e) => {
            cx.eval(&e.expr)?;
            Ok(Flow::Next)
        }
        Stmt::Decl(d) => {
            let v = match &d.init {
                Some(init) => cx.eval(init)?,
                None => Val::Null,
            };
            let top = cx.scopes.last_mut().ok_or_else(|| {
                InterpError::Runtime("no active scope for declaration".to_string())
            })?;
            bind_pattern(&d.pattern, v, top)?;
            Ok(Flow::Next)
        }
        Stmt::Block(b) => exec_block(cx, b, defers),
        Stmt::If(i) => {
            let cond = cx.eval(&i.condition)?;
            if cx.truthy(&cond)? {
                exec_block(cx, &i.then_branch, defers)
            } else if let Some(else_body) = &i.else_body {
                exec_block(cx, else_body, defers)
            } else if let Some(else_if) = &i.else_branch {
                // `else if` chain: evaluate as a nested if-statement.
                exec_stmt(
                    cx,
                    &Stmt::If((**else_if).clone()),
                    defers,
                )
            } else {
                Ok(Flow::Next)
            }
        }
        Stmt::Match(m) => {
            let scrutinee = cx.eval(&m.expr)?;
            exec_match_stmt(cx, &scrutinee, &m.arms, defers)
        }
        Stmt::Switch(sw) => {
            let scrutinee = cx.eval(&sw.scrutinee)?;
            // C-like semantics (see `SwitchStmt` docs): execution jumps to
            // the first matching case (or `default` when nothing matches)
            // and then FALLS THROUGH the remaining bodies in order until a
            // `break`/`return` — `default` trails the cases in the AST.
            let mut start: Option<usize> = None;
            for (idx, case) in sw.cases.iter().enumerate() {
                let mut hit = false;
                for option in &case.exprs {
                    let v = cx.eval(option)?;
                    if scrutinee.equals(&v) {
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    continue;
                }
                if let Some(guard) = &case.guard {
                    let gv = cx.eval(guard)?;
                    if !cx.truthy(&gv)? {
                        continue;
                    }
                }
                start = Some(idx);
                break;
            }
            match start {
                None => {
                    if let Some(default) = &sw.default {
                        exec_block(cx, default, defers)
                    } else {
                        Ok(Flow::Next)
                    }
                }
                Some(first) => {
                    for case in &sw.cases[first..] {
                        match exec_block(cx, &case.body, defers)? {
                            Flow::Next => continue, // fall through
                            other => return Ok(other),
                        }
                    }
                    if let Some(default) = &sw.default {
                        exec_block(cx, default, defers)
                    } else {
                        Ok(Flow::Next)
                    }
                }
            }
        }
        Stmt::For(f) => {
            let iterable = cx.eval(&f.iterable)?;
            let items = expand_iterable(&iterable)?;
            for item in items {
                cx.scopes.push(HashMap::new());
                {
                    let top = cx.scopes.last_mut().ok_or_else(|| {
                        InterpError::Runtime("no active scope for loop variable".to_string())
                    })?;
                    bind_pattern(&f.pattern, item, top)?;
                }
                match exec_block(cx, &f.body, defers)? {
                    Flow::Next | Flow::Continue => {}
                    Flow::Break => {
                        cx.scopes.pop();
                        break;
                    }
                    Flow::Return(v) => {
                        cx.scopes.pop();
                        return Ok(Flow::Return(v));
                    }
                }
                cx.scopes.pop();
            }
            Ok(Flow::Next)
        }
        Stmt::While(w) => loop {
            let cond = cx.eval(&w.condition)?;
            if !cx.truthy(&cond)? {
                break Ok(Flow::Next);
            }
            match exec_block(cx, &w.body, defers)? {
                Flow::Next | Flow::Continue => {}
                Flow::Break => break Ok(Flow::Next),
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
        },
        Stmt::DoWhile(d) => loop {
            match exec_block(cx, &d.body, defers)? {
                Flow::Next | Flow::Continue => {}
                Flow::Break => break Ok(Flow::Next),
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
            let cond = cx.eval(&d.condition)?;
            if !cx.truthy(&cond)? {
                break Ok(Flow::Next);
            }
        },
        Stmt::Loop(l) => loop {
            match exec_block(cx, &l.body, defers)? {
                Flow::Next | Flow::Continue => {}
                Flow::Break => break Ok(Flow::Next),
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
        },
        Stmt::Return(r) => {
            let v = match &r.value {
                Some(e) => cx.eval(e)?,
                None => Val::Unit,
            };
            Ok(Flow::Return(v))
        }
        Stmt::Break(_) => Ok(Flow::Break),
        Stmt::Continue(_) => Ok(Flow::Continue),
        Stmt::Defer(d) => {
            // Defer bodies are blocks in statement position; other shapes
            // evaluate eagerly for their side effects, then do nothing.
            // (Single-expression defers like `defer close()` are uncommon;
            // running them at call-exit would need closures.)
            defers.push(Block {
                statements: vec![Stmt::Expr(lexicon_parser::ExprStmt {
                    expr: d.expr.clone(),
                    span: Default::default(),
                })],
                span: Default::default(),
            });
            Ok(Flow::Next)
        }
        Stmt::AsyncBlock(a) => {
            // `lex run` executes async blocks synchronously.
            cx.eval(&a.body)?;
            Ok(Flow::Next)
        }
    }
}

fn expand_iterable(v: &Val) -> IResult<Vec<Val>> {
    match v {
        Val::List(items) => Ok(items.clone()),
        Val::Tuple(items) => Ok(items.clone()),
        Val::Range { start, end, inclusive } => {
            let mut out = Vec::new();
            if *inclusive {
                let mut i = *start;
                while i <= *end {
                    out.push(Val::Int(i));
                    if i == i64::MAX {
                        break;
                    }
                    i += 1;
                    if out.len() > 1_000_000 {
                        return runtime("range too large to iterate");
                    }
                }
            } else {
                let mut i = *start;
                while i < *end {
                    out.push(Val::Int(i));
                    if i == i64::MAX {
                        break;
                    }
                    i += 1;
                    if out.len() > 1_000_000 {
                        return runtime("range too large to iterate");
                    }
                }
            }
            Ok(out)
        }
        Val::Str(s) => Ok(s.chars().map(|c| Val::Str(c.to_string())).collect()),
        other => runtime(format!("cannot iterate over {}", other.type_name())),
    }
}

/// Statement-`match`: arms run in order; the first matching arm's
/// expression body evaluates for its side effects.
fn exec_match_stmt(
    cx: &mut Cx,
    scrutinee: &Val,
    arms: &[MatchArm],
    _defers: &mut Vec<Block>,
) -> IResult<Flow> {
    for arm in arms {
        let mut arm_scope = HashMap::new();
        if !match_pattern(&arm.pattern, scrutinee, &mut arm_scope)? {
            continue;
        }
        cx.scopes.push(arm_scope);
        if let Some(guard) = &arm.guard {
            let ok = cx.eval(guard).and_then(|v| cx.truthy(&v))?;
            if !ok {
                cx.scopes.pop();
                continue;
            }
        }
        // Arm bodies are expressions: evaluate for effects, then done.
        // (`return`/`break` cannot arise from expression evaluation.)
        let result = cx.eval(&arm.body);
        cx.scopes.pop();
        result?;
        return Ok(Flow::Next);
    }
    Ok(Flow::Next)
}

fn eval_match_expr(cx: &mut Cx, scrutinee: Val, arms: &[MatchArm]) -> IResult<Val> {
    for arm in arms {
        let mut arm_scope = HashMap::new();
        if !match_pattern(&arm.pattern, &scrutinee, &mut arm_scope)? {
            continue;
        }
        cx.scopes.push(arm_scope);
        if let Some(guard) = &arm.guard {
            let ok = cx.eval(guard).and_then(|v| cx.truthy(&v))?;
            if !ok {
                cx.scopes.pop();
                continue;
            }
        }
        let v = cx.eval(&arm.body)?;
        cx.scopes.pop();
        return Ok(v);
    }
    runtime("no match arm matched (non-exhaustive match at runtime)")
}

fn match_pattern(
    pattern: &Pattern,
    value: &Val,
    scope: &mut HashMap<String, Val>,
) -> IResult<bool> {
    match pattern {
        Pattern::Wildcard(_) => Ok(true),
        Pattern::Ident(id) => {
            if id.text == "_" {
                Ok(true)
            } else {
                scope.insert(id.text.clone(), value.clone());
                Ok(true)
            }
        }
        Pattern::Literal(lit) => {
            let expected = literal_val(lit)?;
            Ok(value.equals(&expected))
        }        Pattern::Tuple(parts) => match value {
            Val::Tuple(items) | Val::List(items) => {
                if items.len() != parts.len() {
                    return Ok(false);
                }
                for (p, v) in parts.iter().zip(items.iter()) {
                    if !match_pattern(p, v, scope)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        },
        Pattern::Or(alts) => {
            for alt in alts {
                let mut tmp = HashMap::new();
                if match_pattern(alt, value, &mut tmp)? {
                    scope.extend(tmp);
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => unsupported("complex match pattern"),
    }
}

fn literal_val(lit: &Literal) -> IResult<Val> {
    match lit {
        Literal::Bool(b) => Ok(Val::Bool(*b)),
        Literal::Int(n, _) => Ok(Val::Int(*n)),
        Literal::Float(x, _) => Ok(Val::Float(*x)),
        Literal::String(s) => Ok(Val::Str(s.clone())),
        Literal::Char(c) => Ok(Val::Char(*c)),
        // Array literal patterns only ever match the empty list here;
        // element patterns stay unsupported (handled by the caller).
        Literal::Array(items) => {
            if items.is_empty() {
                Ok(Val::List(Vec::new()))
            } else {
                unsupported("non-empty array literal pattern")
            }
        }
        Literal::Tuple(_) => unsupported("tuple literal pattern"),
        Literal::Unit => Ok(Val::Unit),
        Literal::Null => Ok(Val::Null),
    }
}

impl Cx {
    fn eval(&mut self, expr: &Expr) -> IResult<Val> {
        self.step()?;
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Bool(b) => Ok(Val::Bool(*b)),
                Literal::Int(n, _) => Ok(Val::Int(*n)),
                Literal::Float(x, _) => Ok(Val::Float(*x)),
                Literal::String(s) => Ok(Val::Str(s.clone())),
                Literal::Char(c) => Ok(Val::Char(*c)),
                Literal::Array(items) => {
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        out.push(self.eval(item)?);
                    }
                    Ok(Val::List(out))
                }
                Literal::Tuple(items) => {
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        out.push(self.eval(item)?);
                    }
                    Ok(Val::Tuple(out))
                }
                Literal::Unit => Ok(Val::Unit),
                Literal::Null => Ok(Val::Null),
            },
            Expr::Ident(id) => self
                .lookup(&id.text)
                .ok_or_else(|| InterpError::Runtime(format!("undefined variable `{}`", id.text))),
            Expr::Tuple(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.eval(item)?);
                }
                Ok(Val::Tuple(out))
            }
            Expr::InterpolatedString(parts) => {
                let mut s = String::new();
                for part in parts {
                    match part {
                        InterpolatedPart::Text(t) => s.push_str(t),
                        InterpolatedPart::Expr(e) => s.push_str(&self.eval(e)?.display()),
                    }
                }
                Ok(Val::Str(s))
            }
            Expr::Range(start, end, inclusive) => {
                let a = self.eval(start)?;
                let b = self.eval(end)?;
                match (a, b) {
                    (Val::Int(x), Val::Int(y)) => Ok(Val::Range {
                        start: x,
                        end: y,
                        inclusive: *inclusive,
                    }),
                    _ => runtime("range bounds must be integers"),
                }
            }
            Expr::NullCoalesce(a, b) => match self.eval(a)? {
                Val::Null => self.eval(b),
                v => Ok(v),
            },
            Expr::Unary(u) => {
                let v = self.eval(&u.operand)?;
                match u.op {
                    UnOp::Neg => match v {
                        Val::Int(n) => Ok(Val::Int(-n)),
                        Val::Float(x) => Ok(Val::Float(-x)),
                        other => runtime(format!("cannot negate {}", other.type_name())),
                    },
                    UnOp::Not => match v {
                        Val::Bool(b) => Ok(Val::Bool(!b)),
                        other => runtime(format!("cannot apply `!` to {}", other.type_name())),
                    },
                    UnOp::BitNot => match v {
                        Val::Int(n) => Ok(Val::Int(!n)),
                        other => runtime(format!("cannot apply `~` to {}", other.type_name())),
                    },
                    // `&x` / `&mut x` borrow — identity until borrowck
                    // lands (no move semantics in the tree interpreter).
                    // `*x` deref — identity for the same reason.
                    // `x?` unwrap — `null` propagates, else identity.
                    UnOp::Ref | UnOp::RefMut | UnOp::Deref => Ok(v),
                    UnOp::Question => match v {
                        Val::Null => Ok(Val::Null),
                        other => Ok(other),
                    },
                }
            }
            Expr::Binary(b) => self.eval_binary(b),
            Expr::Cast(c) => {
                let v = self.eval(&c.expr)?;
                Ok(cast_val(v, &c.ty))
            }
            Expr::FieldAccess(f) => {
                // `mod::CONST` / `mod::global`: module globals loaded from
                // `.lex` files are readable exactly like module functions
                // are callable (Fase 0 follow-up — unlocks `math::Pi`).
                if let Expr::Ident(id) = &*f.object {
                    if let Some(m) = self.mods.get(&id.text) {
                        if !m.public.contains(&f.field.text) {
                            return runtime(format!("`{}::{}` is not public", id.text, f.field.text));
                        }
                        if let Some(v) = m.globals.get(&f.field.text) {
                            return Ok(v.clone());
                        }
                        if m.funcs.contains_key(&f.field.text) {
                            return runtime(format!(
                                "`{}::{}` is a function, call it with `()`",
                                id.text, f.field.text
                            ));
                        }
                        return runtime(format!(
                            "module `{}` has no item `{}`",
                            id.text, f.field.text
                        ));
                    }
                }
                let obj = self.eval(&f.object)?;
                self.field_of(obj, &f.field.text)
            }
            Expr::OptionalFieldAccess(o) => {
                if let Expr::Ident(id) = &*o.object {
                    if let Some(m) = self.mods.get(&id.text) {
                        if !m.public.contains(&o.field.text) {
                            return runtime(format!("`{}::{}` is not public", id.text, o.field.text));
                        }
                        return Ok(m.globals.get(&o.field.text).cloned().unwrap_or(Val::Null));
                    }
                }
                let obj = self.eval(&o.object)?;
                match obj {
                    Val::Null => Ok(Val::Null),
                    other => self.field_of(other, &o.field.text),
                }
            }
            Expr::Index(ix) => {
                let obj = self.eval(&ix.object)?;
                let idx = self.eval(&ix.index)?;
                match (obj, idx) {
                    (Val::List(items), Val::Int(i)) => {
                        if i < 0 || (i as usize) >= items.len() {
                            return runtime(format!(
                                "index {} out of bounds (len {})",
                                i,
                                items.len()
                            ));
                        }
                        Ok(items[i as usize].clone())
                    }
                    (Val::Str(s), Val::Int(i)) => {
                        let chars: Vec<char> = s.chars().collect();
                        if i < 0 || (i as usize) >= chars.len() {
                            return runtime("string index out of bounds");
                        }
                        Ok(Val::Str(chars[i as usize].to_string()))
                    }
                    (obj, _) => runtime(format!("cannot index into {}", obj.type_name())),
                }
            }
            Expr::New(n) => {
                let type_name = match &n.ty {
                    lexicon_parser::Type::Path(p) => {
                        p.segments.last().map(|s| s.text.clone()).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                let mut args = Vec::with_capacity(n.args.len());
                for arg in &n.args {
                    args.push(self.eval(arg)?);
                }
                if type_name.is_empty() {
                    return unsupported("struct literal with complex type");
                }
                if let Some(decl) = self.user_struct(&type_name) {
                    let mut fields = Vec::new();
                    for (i, field) in decl.fields.iter().enumerate() {
                        let v = args.get(i).cloned().unwrap_or(Val::Null);
                        fields.push((field.name.text.clone(), v));
                    }
                    Ok(Val::Struct { name: type_name, fields })
                } else {
                    // Unknown struct (generic/external): keep positional values
                    // printable instead of failing the whole run.
                    let fields = args
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| (format!("field{}", i), v))
                        .collect();
                    Ok(Val::Struct { name: type_name, fields })
                }
            }
            Expr::Call(c) => self.eval_call(&c.callee, &c.args),
            Expr::MethodCall(m) => self.eval_method(&m.object, &m.method.text, &m.args),
            Expr::If(i) => {
                let cond = self.eval(&i.condition)?;
                if self.truthy(&cond)? {
                    self.eval(&i.then_expr)
                } else if let Some(else_expr) = &i.else_expr {
                    self.eval(else_expr)
                } else {
                    Ok(Val::Unit)
                }
            }
            Expr::Match(m) => {
                let scrutinee = self.eval(&m.expr)?;
                eval_match_expr(self, scrutinee, &m.arms)
            }
            Expr::Block(b) => {
                self.scopes.push(HashMap::new());
                let mut flow = Flow::Next;
                let mut defers: Vec<Block> = Vec::new();
                for stmt in &b.statements {
                    flow = exec_stmt(self, stmt, &mut defers)?;
                    if !matches!(flow, Flow::Next) {
                        break;
                    }
                }
                self.scopes.pop();
                match flow {
                    Flow::Next => Ok(Val::Unit),
                    // `return`/`break` inside a block-expression: the value
                    // path cannot represent control flow — surface it.
                    Flow::Return(v) => Ok(v),
                    Flow::Break => runtime("`break` outside of a loop"),
                    Flow::Continue => runtime("`continue` outside of a loop"),
                }
            }
            Expr::Await(a) => self.eval(&a.expr),
            Expr::UnsafeBlock(b, _) => {
                self.scopes.push(HashMap::new());
                let mut defers: Vec<Block> = Vec::new();
                for stmt in &b.statements {
                    match exec_stmt(self, stmt, &mut defers)? {
                        Flow::Next => {}
                        Flow::Return(v) => {
                            self.scopes.pop();
                            return Ok(v);
                        }
                        Flow::Break => {
                            self.scopes.pop();
                            return runtime("`break` outside of a loop");
                        }
                        Flow::Continue => {
                            self.scopes.pop();
                            return runtime("`continue` outside of a loop");
                        }
                    }
                }
                self.scopes.pop();
                Ok(Val::Unit)
            }
            Expr::Try(_) => unsupported("`try`/`catch`"),
            Expr::Lambda(l) => {
                // First-class lambda value: snapshot the visible bindings
                // so the function keeps working when it escapes into
                // another module (`slices::Map(xs, |x| x * k)`).
                let mut captured = HashMap::new();
                for scope in self.scopes.iter() {
                    for (k, v) in scope.iter() {
                        captured.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                }
                for (k, v) in self.globals.iter() {
                    captured.entry(k.clone()).or_insert_with(|| v.clone());
                }
                Ok(Val::Func(FuncVal {
                    params: l.params.iter().map(|p| p.name.text.clone()).collect(),
                    body: l.body.clone(),
                    captured,
                }))
            }
            Expr::Spawn(_) => unsupported("`spawn`"),
            Expr::Select(_) => unsupported("`select`"),
            // `Some(v)` carries the value; `None` is `null`
            // (the tree interpreter has no tagged-Option runtime).
            Expr::Option(opt) => match opt {
                lexicon_parser::OptionLiteral::Some(e) => self.eval(e),
                lexicon_parser::OptionLiteral::None(_) => Ok(Val::Null),
            },
            Expr::Destructuring(_) => unsupported("destructuring expression"),
            Expr::MacroCall(_) => unsupported("macro call"),
            Expr::Attribute(_) => unsupported("attribute expression"),
            Expr::With(_) => unsupported("`with` expression"),
            Expr::Spread(_, _) => unsupported("spread expression"),
            Expr::AsyncBlock(_) => unsupported("async block value"),
        }
    }

    fn field_of(&self, obj: Val, field: &str) -> IResult<Val> {
        match obj {
            Val::Struct { fields, .. } => fields
                .into_iter()
                .find(|(k, _)| k == field)
                .map(|(_, v)| v)
                .ok_or_else(|| InterpError::Runtime(format!("no field `{}` on value", field))),
            Val::Tuple(items) => match field.parse::<usize>() {
                Ok(i) if i < items.len() => Ok(items[i].clone()),
                _ => runtime(format!("no field `{}` on tuple", field)),
            },
            other => runtime(format!("no field `{}` on {}", field, other.type_name())),
        }
    }

    fn eval_binary(&mut self, b: &lexicon_parser::BinaryExpr) -> IResult<Val> {
        // Short-circuiting logical operators evaluate the RHS lazily.
        match b.op {
            BinOp::AndAnd | BinOp::AmpAmp => {
                let l = self.eval(&b.lhs)?;
                if !self.truthy(&l)? {
                    return Ok(Val::Bool(false));
                }
                let r = self.eval(&b.rhs)?;
                return Ok(Val::Bool(self.truthy(&r)?));
            }
            BinOp::OrOr | BinOp::PipePipe => {
                let l = self.eval(&b.lhs)?;
                if self.truthy(&l)? {
                    return Ok(Val::Bool(true));
                }
                let r = self.eval(&b.rhs)?;
                return Ok(Val::Bool(self.truthy(&r)?));
            }
            // `=` parses as `AddEq` (placeholder); treat it as assignment.
            // (`+=` never reaches the interpreter as `AddEq`.)
            BinOp::AddEq => {
                // Append fast path: `x = x + part [+ part…]` appends each
                // part to the bound string IN PLACE (no full-string clone),
                // turning string-building loops from O(n²) into O(n).
                // Falls back to generic eval+assign on any shape mismatch.
                if let Expr::Ident(id) = &*b.lhs {
                    if let Some(v) = self.try_append(&id.text, &b.rhs)? {
                        return Ok(v);
                    }
                }
                let v = self.eval(&b.rhs)?;
                self.assign_target(&b.lhs, v.clone())?;
                return Ok(v);
            }
            BinOp::SubEq | BinOp::MulEq | BinOp::DivEq | BinOp::ModEq => {
                let cur = self.eval(&b.lhs)?;
                let rhs = self.eval(&b.rhs)?;
                let v = apply_arith(
                    match b.op {
                        BinOp::SubEq => BinOp::Sub,
                        BinOp::MulEq => BinOp::Mul,
                        BinOp::DivEq => BinOp::Div,
                        _ => BinOp::Mod,
                    },
                    cur,
                    rhs,
                )?;
                self.assign_target(&b.lhs, v.clone())?;
                return Ok(v);
            }
            BinOp::Pipe => {
                // `value |> f`: single-argument call when `f` names a function.
                let lhs = self.eval(&b.lhs)?;
                match &*b.rhs {
                    Expr::Ident(id) => {
                        if let Some((func, home)) = self.user_fn(&id.text) {
                            return call_user_home(self, &func, vec![lhs], home);
                        }
                        return self.call_named(&id.text, vec![lhs]);
                    }
                    // `value |> mod::Fn` — the qualified form of the above.
                    Expr::FieldAccess(f) => match &*f.object {
                        Expr::Ident(module) => {
                            return self.call_module(&module.text, &f.field.text, vec![lhs]);
                        }
                        _ => return unsupported("pipe into complex receiver"),
                    },
                    Expr::Call(c) => {
                        let mut vals = vec![lhs];
                        for a in &c.args {
                            vals.push(self.eval(a)?);
                        }
                        return self.eval_call_vals(&c.callee, vals);
                    }
                    _ => return unsupported("pipe into non-function"),
                }
            }
            _ => {}
        }
        let l = self.eval(&b.lhs)?;
        let r = self.eval(&b.rhs)?;
        match b.op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                apply_arith(b.op.clone(), l, r)
            }
            BinOp::EqEq => Ok(Val::Bool(l.equals(&r))),
            BinOp::Neq => Ok(Val::Bool(!l.equals(&r))),
            BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => compare(b.op.clone(), &l, &r),
            BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                apply_arith(b.op.clone(), l, r)
            }
            _ => unsupported("binary operator"),
        }
    }

    /// Append fast path for `name = name + part [+ part…]`: evaluates each
    /// part left-to-right and pushes it onto the bound string without ever
    /// cloning the buffer. Returns `Ok(None)` when the shape doesn't apply
    /// (caller falls back to generic eval+assign) and `Ok(Some(result))`
    /// after a successful in-place append.
    fn try_append(&mut self, name: &str, expr: &Expr) -> IResult<Option<Val>> {
        // Collect the `+` chain parts; the leftmost leaf must be `name`.
        let mut parts: Vec<&Expr> = Vec::new();
        let mut node = expr;
        loop {
            match node {
                Expr::Binary(b) if b.op == BinOp::Add => {
                    parts.push(&b.rhs);
                    node = &b.lhs;
                }
                Expr::Ident(id) if id.text == name => break,
                _ => return Ok(None),
            }
        }
        // The binding must already hold a string (unbound or non-string
        // falls back — e.g. `x = x + 1` on ints keeps exact semantics).
        if !self.bound_is_str(name) {
            return Ok(None);
        }
        for part in parts.iter().rev() {
            self.step()?;
            let v = self.eval(part)?;
            if !self.append_to(name, &v.display())? {
                return Ok(None);
            }
        }
        Ok(self.lookup(name))
    }

    fn bound_is_str(&self, name: &str) -> bool {
        if let Some(v) = self.lookup(name) {
            return matches!(v, Val::Str(_));
        }
        false
    }

    /// Pushes `s` onto the string bound to `name`. `false` = shape changed
    /// under us (caller falls back).
    fn append_to(&mut self, name: &str, s: &str) -> IResult<bool> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(v) = scope.get_mut(name) {
                if let Val::Str(buf) = v {
                    buf.push_str(s);
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        if let Some(v) = self.globals.get_mut(name) {
            if let Val::Str(buf) = v {
                buf.push_str(s);
                return Ok(true);
            }
            return Ok(false);
        }
        Ok(false)
    }

    fn assign_target(&mut self, target: &Expr, val: Val) -> IResult<()> {
        match target {
            Expr::Ident(id) => {
                self.assign(&id.text, val);
                Ok(())
            }            Expr::FieldAccess(f) => match &*f.object {
                Expr::Ident(id) => {
                    let mut obj = self.lookup(&id.text).ok_or_else(|| {
                        InterpError::Runtime(format!("undefined variable `{}`", id.text))
                    })?;
                    match &mut obj {
                        Val::Struct { fields, .. } => {
                            if let Some(slot) = fields.iter_mut().find(|(k, _)| k == &f.field.text) {
                                slot.1 = val;
                                self.assign(&id.text, obj);
                                Ok(())
                            } else {
                                runtime(format!("no field `{}` to assign", f.field.text))
                            }
                        }
                        other => runtime(format!("cannot assign field on {}", other.type_name())),
                    }
                }
                _ => unsupported("assignment to complex field path"),
            },
            Expr::Index(ix) => {
                let idx = self.eval(&ix.index)?;
                match &*ix.object {
                    Expr::Ident(id) => {
                        let mut obj = self.lookup(&id.text).ok_or_else(|| {
                            InterpError::Runtime(format!("undefined variable `{}`", id.text))
                        })?;
                        match (&mut obj, idx) {
                            (Val::List(items), Val::Int(i)) => {
                                if i < 0 || (i as usize) >= items.len() {
                                    return runtime("index out of bounds");
                                }
                                items[i as usize] = val;
                                self.assign(&id.text, obj);
                                Ok(())
                            }
                            _ => runtime("cannot assign through this index"),
                        }
                    }
                    _ => unsupported("assignment to complex index path"),
                }
            }
            _ => runtime("invalid assignment target"),
        }
    }

    /// Call by already-evaluated argument values (used by the pipe operator).
    fn eval_call_vals(&mut self, callee: &Expr, args: Vec<Val>) -> IResult<Val> {
        match callee {
            Expr::Ident(id) => {
                if let Some(Val::Func(f)) = self.lookup(&id.text) {
                    let f = f.clone();
                    return self.call_func(&f, args);
                }
                self.call_named(&id.text, args)
            }
            Expr::FieldAccess(f) => match &*f.object {
                Expr::Ident(module) => self.call_module(&module.text, &f.field.text, args),
                _ => unsupported("call on complex receiver"),
            },
            _ => unsupported("call callee shape"),
        }
    }

    fn call_func(&mut self, func: &FuncVal, args: Vec<Val>) -> IResult<Val> {
        call_lambda(self, func, args)
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> IResult<Val> {
        match callee {
            Expr::Ident(id) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a)?);
                }
                // Variable holding a lambda shadows same-named functions
                // (enables `f(x)` where `f` is a parameter/let binding).
                if let Some(Val::Func(f)) = self.lookup(&id.text) {
                    let f = f.clone();
                    return self.call_func(&f, vals);
                }
                self.call_named(&id.text, vals)
            }
            Expr::FieldAccess(f) => match &*f.object {
                Expr::Ident(module) => {
                    let mut vals = Vec::with_capacity(args.len());
                    for a in args {
                        vals.push(self.eval(a)?);
                    }
                    // `value.field(args)` where `value` is a bound variable
                    // holding a struct with a function field (e.g. the
                    // `less` comparator stored in `heap::Heap`).
                    if self.mods.get(&module.text).is_none() {
                        if let Some(obj) = self.lookup(&module.text) {
                            if let Val::Struct { fields, .. } = obj {
                                if let Some((_, Val::Func(g))) = fields
                                    .iter()
                                    .find(|(k, _)| k == &f.field.text)
                                {
                                    let g = g.clone();
                                    return self.call_func(&g, vals);
                                }
                            }
                        }
                    }
                    self.call_module(&module.text, &f.field.text, vals)
                }
                _ => unsupported("module call on complex receiver"),
            },
            _ => unsupported("call callee shape"),
        }
    }

    fn call_named(&mut self, name: &str, args: Vec<Val>) -> IResult<Val> {
        match name {
            "print" | "println" => {
                let line = args.iter().map(|v| v.display()).collect::<Vec<_>>().join(" ");
                if name == "println" {
                    self.push_output(&line)?;
                    self.push_output("\n")?;
                } else {
                    self.push_output(&line)?;
                }
                Ok(Val::Unit)
            }
            "assert" => {
                let cond = args.first().cloned().unwrap_or(Val::Bool(false));
                let msg = args.get(1).map(|v| v.display());
                if self.truthy(&cond)? {
                    Ok(Val::Unit)
                } else {
                    runtime(msg.unwrap_or_else(|| "assertion failed".to_string()))
                }
            }
            "panic" => {
                let msg = args.first().map(|v| v.display()).unwrap_or_else(|| "panic".to_string());
                runtime(msg)
            }
            "inspect" => {
                let s = args.first().map(|v| v.inspect()).unwrap_or_default();
                Ok(Val::Str(s))
            }
            "typeOf" | "typeof" | "type" => {
                // Runtime reflection: name of the value's type as a string
                // (`typeOf(1)` -> `"int"`, `typeOf("s")` -> `"String"`,
                // structs report their declared name). Arity mirrors other
                // single-arg builtins: exactly one argument.
                if args.len() != 1 {
                    return runtime(format!(
                        "`{}` takes exactly one argument, found {}",
                        name,
                        args.len()
                    ));
                }
                let v = &args[0];
                Ok(Val::Str(v.type_of()))
            }
            _ => {
                let (func, home) = self.user_fn(name).ok_or_else(|| {
                    InterpError::Runtime(format!("undefined function `{}`", name))
                })?;
                call_user_home(self, &func, args, home)
            }
        }
    }

    // ------------------------------------------------------------------
    // Motor gráfico nativo (`Window`/`Canvas`/`Input`/`Gpu` + widgets).
    // feature `gui`: janelas reais desenhadas pelo renderer wgpu do eframe
    // (Vulkan / DirectX 12 / OpenGL). Fora da feature, as chamadas caem no
    // caminho antigo de mock (`Unsupported`).
    // ------------------------------------------------------------------
    #[cfg(feature = "gui")]
    fn gfx_module(&mut self, module: &str, member: &str, args: &[Val]) -> Option<IResult<Val>> {
        use crate::gfx::Widget;

        let disp = |v: &Val| v.display();
        let num = |v: &Val| match v {
            Val::Int(n) => *n as f64,
            Val::Float(x) => *x,
            _ => 0.0,
        };
        let wid = |v: &Val| -> Option<i64> {
            if let Val::Struct { fields, .. } = v {
                if let Some((_, Val::Int(n))) = fields.iter().find(|(k, _)| k == "id") {
                    return Some(*n);
                }
            }
            None
        };
        let handle_val = |name: &str, id: i64| Val::Struct {
            name: name.to_string(),
            fields: vec![("id".to_string(), Val::Int(id))],
        };
        fn col(args: &[Val], i: usize) -> [f32; 4] {
            let f = |j: usize| -> f32 {
                match args.get(i + j) {
                    Some(Val::Int(n)) => *n as f32,
                    Some(Val::Float(x)) => *x as f32,
                    _ => 0.0,
                }
            };
            [f(0), f(1), f(2), if args.len() > i + 3 { f(3) } else { 1.0 }]
        }
        let no_window = || runtime("espera um handle de janela (Window::create)");

        match (module, member) {
            ("Window", "create") => {
                let (mut title, mut w, mut h) = ("Janela".to_string(), 800.0f64, 600.0f64);
                if let Some(Val::Struct { fields, .. }) = args.first() {
                    for (k, v) in fields {
                        match (k.as_str(), v) {
                            ("title", Val::Str(s)) => title = s.clone(),
                            ("width", _) => w = num(v),
                            ("height", _) => h = num(v),
                            _ => {}
                        }
                    }
                }
                if w <= 0.0 || w > 16_384.0 || h <= 0.0 || h > 16_384.0 {
                    return Some(runtime(format!(
                        "Window::create: tamanho inválido ({}x{})",
                        w, h
                    )));
                }
                let id = crate::gfx::window_create(title, w as f32, h as f32);
                Some(Ok(handle_val("Window", id)))
            }
            ("Window", "should_close") | ("Window", "shouldClose") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Bool(crate::gfx::window_should_close(id))))
            }
            ("Window", "present") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::window_present(id);
                Some(Ok(Val::Unit))
            }
            ("Window", "poll") | ("Window", "show") => Some(Ok(Val::Unit)),
            ("Window", "close") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::window_close(id);
                Some(Ok(Val::Unit))
            }
            ("Window", "set_title") | ("Window", "setTitle") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::window_set_title(id, args.get(1).map(disp).unwrap_or_default());
                Some(Ok(Val::Unit))
            }
            ("Window", "set_size") | ("Window", "setSize") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::window_set_size(
                    id,
                    args.get(1).map(|v| num(v) as f32).unwrap_or(800.0),
                    args.get(2).map(|v| num(v) as f32).unwrap_or(600.0),
                );
                Some(Ok(Val::Unit))
            }
            ("Window", "width") | ("Window", "w") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Int(crate::gfx::window_width(id) as i64)))
            }
            ("Window", "height") | ("Window", "h") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Int(crate::gfx::window_height(id) as i64)))
            }
            ("Window", "backend") | ("Gpu", "backend") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Str(crate::gfx::window_backend(id))))
            }
            ("Canvas", "clear") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::canvas_clear(id, col(args, 1));
                Some(Ok(Val::Unit))
            }
            ("Canvas", "fill_rect") | ("Canvas", "fillRect") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::canvas_fill_rect(
                    id,
                    args.get(1).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(2).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(3).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(4).map(|v| num(v) as f32).unwrap_or(0.0),
                    col(args, 5),
                );
                Some(Ok(Val::Unit))
            }
            ("Canvas", "fill_circle") | ("Canvas", "fillCircle") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::canvas_fill_circle(
                    id,
                    args.get(1).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(2).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(3).map(|v| num(v) as f32).unwrap_or(0.0),
                    col(args, 4),
                );
                Some(Ok(Val::Unit))
            }
            ("Canvas", "line") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::canvas_line(
                    id,
                    args.get(1).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(2).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(3).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(4).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(5).map(|v| num(v) as f32).unwrap_or(2.0),
                    col(args, 6),
                );
                Some(Ok(Val::Unit))
            }
            ("Canvas", "text") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                crate::gfx::canvas_text(
                    id,
                    args.get(1).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(2).map(|v| num(v) as f32).unwrap_or(0.0),
                    args.get(3).map(disp).unwrap_or_default(),
                    args.get(4).map(|v| num(v) as f32).unwrap_or(16.0),
                    col(args, 5),
                );
                Some(Ok(Val::Unit))
            }
            ("Input", "key_down") | ("Input", "keyDown") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Bool(crate::gfx::input_key_down(
                    id,
                    &args.get(1).map(disp).unwrap_or_default(),
                ))))
            }
            ("Input", "mouse_x") | ("Input", "mouseX") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Float(crate::gfx::input_mouse_x(id) as f64)))
            }
            ("Input", "mouse_y") | ("Input", "mouseY") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Float(crate::gfx::input_mouse_y(id) as f64)))
            }
            ("Input", "mouse_down") | ("Input", "mouseDown") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(no_window());
                };
                Some(Ok(Val::Bool(crate::gfx::input_mouse_down(id))))
            }
            ("Input", "clicked") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(runtime("Input::clicked(widget) espera um handle de widget"));
                };
                Some(Ok(Val::Bool(crate::gfx::input_clicked(id))))
            }
            ("Input", "value") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(runtime("Input::value(widget) espera um handle de widget"));
                };
                Some(Ok(Val::Str(crate::gfx::input_value(id))))
            }
            ("Input", "checked") => {
                let Some(id) = args.first().and_then(wid) else {
                    return Some(runtime("Input::checked(widget) espera um handle de widget"));
                };
                Some(Ok(Val::Bool(crate::gfx::input_checked(id))))
            }
            ("Label", "create") => {
                let id = crate::gfx::widget_new(Widget::Label {
                    text: args.first().map(disp).unwrap_or_default(),
                });
                Some(Ok(handle_val("Label", id)))
            }
            ("Button", "create") => {
                let id = crate::gfx::widget_new(Widget::Button {
                    label: args.first().map(disp).unwrap_or_default(),
                });
                Some(Ok(handle_val("Button", id)))
            }
            ("TextField", "create") => {
                let id = crate::gfx::widget_new(Widget::TextField {
                    placeholder: args.first().map(disp).unwrap_or_default(),
                });
                Some(Ok(handle_val("TextField", id)))
            }
            ("Checkbox", "create") => {
                let id = crate::gfx::widget_new(Widget::Checkbox {
                    label: args.first().map(disp).unwrap_or_default(),
                });
                Some(Ok(handle_val("Checkbox", id)))
            }
            ("MenuBar", "new") => {
                let id = crate::gfx::widget_new(Widget::MenuBar);
                Some(Ok(handle_val("MenuBar", id)))
            }
            ("Menu", "new") => {
                let id = crate::gfx::widget_new(Widget::Menu {
                    title: args.first().map(disp).unwrap_or_default(),
                });
                Some(Ok(handle_val("Menu", id)))
            }
            ("MenuItem", "new") => {
                let id = crate::gfx::widget_new(Widget::MenuItem {
                    label: args.first().map(disp).unwrap_or_default(),
                    shortcut: args.get(1).map(disp).unwrap_or_default(),
                });
                Some(Ok(handle_val("MenuItem", id)))
            }
            _ => None,
        }
    }

    /// Métodos sobre handles gráficos (`window.setSize(...)`, `bar.add(...)`) —
    /// roteados pelo `eval_method` quando o receptor é um `Struct` cujo nome
    /// é um tipo gráfico conhecido.
    #[cfg(feature = "gui")]
    fn gfx_method(&mut self, kind: &str, id: i64, method: &str, args: &[Val]) -> IResult<Val> {
        use crate::gfx as g;
        let wid = |v: &Val| -> Option<i64> {
            if let Val::Struct { fields, .. } = v {
                if let Some((_, Val::Int(n))) = fields.iter().find(|(k, _)| k == "id") {
                    return Some(*n);
                }
            }
            None
        };
        match (kind, method) {
            ("Window", "add") => {
                let Some(w) = args.first().and_then(wid) else {
                    return runtime("window.add(widget) espera um handle de widget");
                };
                g::widget_add(id, w);
                Ok(Val::Unit)
            }
            ("Window", "setMenuBar") | ("Window", "set_menu_bar") => {
                let Some(bar) = args.first().and_then(wid) else {
                    return runtime("window.setMenuBar(bar) espera um handle de MenuBar");
                };
                g::window_set_menubar(id, bar);
                Ok(Val::Unit)
            }
            ("Window", "setSize") | ("Window", "set_size") => {
                g::window_set_size(
                    id,
                    args.first().map(|v| match v {
                        Val::Int(n) => *n as f32,
                        Val::Float(x) => *x as f32,
                        _ => 800.0,
                    })
                    .unwrap_or(800.0),
                    args.get(1).map(|v| match v {
                        Val::Int(n) => *n as f32,
                        Val::Float(x) => *x as f32,
                        _ => 600.0,
                    })
                    .unwrap_or(600.0),
                );
                Ok(Val::Unit)
            }
            ("Window", "setTitle") | ("Window", "set_title") => {
                g::window_set_title(
                    id,
                    args.first().map(|v| v.display()).unwrap_or_default(),
                );
                Ok(Val::Unit)
            }
            ("Window", "show") | ("Window", "poll") => Ok(Val::Unit),
            ("Window", "present") => {
                g::window_present(id);
                Ok(Val::Unit)
            }
            ("Window", "shouldClose") | ("Window", "should_close") => {
                Ok(Val::Bool(g::window_should_close(id)))
            }
            ("Window", "close") => {
                g::window_close(id);
                Ok(Val::Unit)
            }
            ("Window", "backend") => Ok(Val::Str(g::window_backend(id))),
            ("Window", "width") => Ok(Val::Int(g::window_width(id) as i64)),
            ("Window", "height") => Ok(Val::Int(g::window_height(id) as i64)),
            // --- Widgets: setText/setPlaceholder/setChecked/getText/... ---
            ("Label", "setText") | ("Label", "set_text")
            | ("Button", "setText") | ("Button", "set_text")
            | ("TextField", "setText") | ("TextField", "set_text") => {
                let s = args.first().map(|v| v.display()).unwrap_or_default();
                if !g::widget_set_text(id, &s) {
                    return runtime(format!("widget #{} não aceita setText", id));
                }
                Ok(Val::Unit)
            }
            ("TextField", "setPlaceholder") | ("TextField", "set_placeholder") => {
                let s = args.first().map(|v| v.display()).unwrap_or_default();
                if !g::widget_set_placeholder(id, &s) {
                    return runtime(format!("widget #{} não é um TextField", id));
                }
                Ok(Val::Unit)
            }
            ("Checkbox", "setChecked") | ("Checkbox", "set_checked") => {
                let c = matches!(args.first(), Some(Val::Bool(true)) | Some(Val::Int(1)));
                if !g::widget_set_checked(id, c) {
                    return runtime(format!("widget #{} não é um Checkbox", id));
                }
                Ok(Val::Unit)
            }
            ("TextField", "getText") | ("TextField", "get_text") => {
                Ok(Val::Str(g::widget_text(id).unwrap_or_default()))
            }
            ("Checkbox", "isChecked") | ("Checkbox", "is_checked") => {
                Ok(Val::Bool(g::widget_checked(id).unwrap_or(false)))
            }
            ("MenuBar", "add") | ("Menu", "add") => {
                let Some(child) = args.first().and_then(wid) else {
                    return runtime("bar.add(...) espera um handle de widget");
                };
                g::menu_add(id, child);
                Ok(Val::Unit)
            }
            _ => runtime(format!("`{}` não é um método de {} (motor gráfico)", method, kind)),
        }
    }

    fn call_module(&mut self, module: &str, member: &str, args: Vec<Val>) -> IResult<Val> {
        // Lex stdlib loaded from `.lex` files wins over the Rust builtins —
        // this is what makes `import std::strings; strings::ToUpper(...)`
        // execute real Lex code instead of falling into the mock.
        if let Some(m) = self.mods.get(module) {
            if !m.public.contains(member) {
                return runtime(format!("`{}::{}` is not public", module, member));
            }
            let f = m.funcs.get(member).cloned().ok_or_else(|| {
                InterpError::Runtime(format!("module `{}` has no function `{}`", module, member))
            })?;
            return call_user_home(self, &f, args, Some(module.to_string()));
        }
        // Motor gráfico nativo: janela/canvas/input/widgets reais (feature gui).
        #[cfg(feature = "gui")]
        if let Some(r) = self.gfx_module(module, member, &args) {
            return r;
        }
        let disp = |v: &Val| v.display();
        match (module, member) {
            ("Console", "writeLine") | ("Console", "log") => {
                let line = args.iter().map(disp).collect::<Vec<_>>().join(" ");
                self.push_output(&line)?;
                self.push_output("\n")?;
                Ok(Val::Unit)
            }
            ("Console", "write") => {
                let line = args.iter().map(disp).collect::<Vec<_>>().join(" ");
                self.push_output(&line)?;
                Ok(Val::Unit)
            }
            ("Json", "serialize") | ("Json", "stringify") => {
                let v = args.first().cloned().unwrap_or(Val::Null);
                Ok(Val::Str(v.to_json().to_string()))
            }
            ("Json", "parse") => {
                let s = args.first().map(disp).unwrap_or_default();
                match serde_json::from_str::<serde_json::Value>(&s) {
                    Ok(v) => Ok(json_to_val(&v)),
                    Err(e) => runtime(format!("Json::parse failed: {}", e)),
                }
            }
            ("Json", "valid") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(serde_json::from_str::<serde_json::Value>(&s).is_ok()))
            }
            ("Env", "get") => {
                let k = args.first().map(disp).unwrap_or_default();
                Ok(Val::Str(std::env::var(&k).unwrap_or_else(|_| "NOT_FOUND".to_string())))
            }
            ("Env", "set") => {
                let k = args.first().map(disp).unwrap_or_default();
                let v = args.get(1).map(disp).unwrap_or_default();
                if !k.is_empty() {
                    std::env::set_var(&k, &v);
                }
                Ok(Val::Str(v))
            }
            ("Env", "exists") => {
                let k = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::env::var_os(&k).is_some()))
            }
            // Clock / sleeps (backs the `time` Lex package).
            ("Time", "now_unix_ms") => {
                let ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                Ok(Val::Int(ms))
            }
            ("Time", "now_unix_s") => {
                let s = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                Ok(Val::Int(s))
            }
            ("Time", "sleep") => {
                let ms = args.first().map(|v| v.display().parse::<u64>().unwrap_or(0)).unwrap_or(0);
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(Val::Unit)
            }
            // Filesystem (backs `os`, `io/fs`, `path/filepath`).
            ("File", "read_string") => {
                let p = args.first().map(disp).unwrap_or_default();
                match std::fs::read_to_string(&p) {
                    Ok(s) => Ok(Val::Str(s)),
                    Err(e) => runtime(format!("File::read_string `{}`: {}", p, e)),
                }
            }
            ("File", "write_string") => {
                let p = args.first().map(disp).unwrap_or_default();
                let c = args.get(1).map(disp).unwrap_or_default();
                Ok(Val::Bool(std::fs::write(&p, c).is_ok()))
            }
            ("File", "exists") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::path::Path::new(&p).exists()))
            }
            ("File", "is_dir") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::path::Path::new(&p).is_dir()))
            }
            ("File", "is_file") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::path::Path::new(&p).is_file()))
            }
            ("File", "remove") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::fs::remove_file(&p).is_ok()))
            }
            ("File", "remove_dir") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::fs::remove_dir(&p).is_ok()))
            }
            ("File", "mkdir") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::fs::create_dir(&p).is_ok()))
            }
            ("File", "mkdir_all") => {
                let p = args.first().map(disp).unwrap_or_default();
                Ok(Val::Bool(std::fs::create_dir_all(&p).is_ok()))
            }
            ("File", "read_dir") => {
                let p = args.first().map(disp).unwrap_or_default();
                let mut names = Vec::new();
                if let Ok(rd) = std::fs::read_dir(&p) {
                    for e in rd.flatten() {
                        names.push(Val::Str(e.file_name().to_string_lossy().into_owned()));
                    }
                }
                Ok(Val::List(names))
            }
            ("File", "size") => {
                let p = args.first().map(disp).unwrap_or_default();
                let n = std::fs::metadata(&p).map(|m| m.len() as i64).unwrap_or(-1);
                Ok(Val::Int(n))
            }
            ("File", "getwd") => Ok(Val::Str(
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            )),
            ("File", "temp_dir") => Ok(Val::Str(
                std::env::temp_dir().to_string_lossy().into_owned(),
            )),
            ("File", "home_dir") => Ok(Val::Str(
                std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))
                    .unwrap_or_default(),
            )),
            // Process (backs `os` args/exit/hostname).
            ("Process", "args") => match script_args() {
                Some(argv) => Ok(Val::List(argv.into_iter().map(Val::Str).collect())),
                None => Ok(Val::List(std::env::args().map(Val::Str).collect())),
            },
            ("Process", "exit") => {
                let code = args.first().map(|v| v.display().parse::<i32>().unwrap_or(0)).unwrap_or(0);
                std::process::exit(code);
            }
            ("Process", "hostname") => Ok(Val::Str(
                std::env::var("COMPUTERNAME")
                    .or_else(|_| std::env::var("HOSTNAME"))
                    .unwrap_or_default(),
            )),
            // Spawn a child process and capture its output (backs `os/exec`).
            // Returns `Output { code, stdout, stderr }`; spawn failures
            // report as `code == -1` with the error in `stderr` instead of
            // aborting the run (mirrors Go's `(*Cmd).Output` error value).
            ("Process", "output") => {
                use std::io::Write as _;
                let prog = args.first().map(disp).unwrap_or_default();
                let argv: Vec<String> = match args.get(1) {
                    Some(Val::List(items)) => items.iter().map(|v| v.display()).collect(),
                    Some(other) => vec![other.display()],
                    None => Vec::new(),
                };
                let input = args.get(2).map(disp).unwrap_or_default();
                let mut cmd = std::process::Command::new(&prog);
                cmd.args(&argv);
                let out = if input.is_empty() {
                    cmd.output()
                } else {
                    match cmd
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                    {
                        Ok(mut child) => {
                            if let Some(mut stdin) = child.stdin.take() {
                                let _ = stdin.write_all(input.as_bytes());
                            }
                            child.wait_with_output()
                        }
                        Err(e) => Err(e),
                    }
                };
                match out {
                    Ok(o) => Ok(Val::Struct {
                        name: "Output".to_string(),
                        fields: vec![
                            ("code".to_string(), Val::Int(o.status.code().unwrap_or(-1) as i64)),
                            ("stdout".to_string(), Val::Str(String::from_utf8_lossy(&o.stdout).into_owned())),
                            ("stderr".to_string(), Val::Str(String::from_utf8_lossy(&o.stderr).into_owned())),
                        ],
                    }),
                    Err(e) => Ok(Val::Struct {
                        name: "Output".to_string(),
                        fields: vec![
                            ("code".to_string(), Val::Int(-1)),
                            ("stdout".to_string(), Val::Str(String::new())),
                            ("stderr".to_string(), Val::Str(format!("spawn `{}`: {}", prog, e))),
                        ],
                    }),
                }
            }
            // Shared atomic int64 cells, referenced by id handle (backs
            // `sync/atomic`; single-threaded today, real atomics underneath
            // so semantics survive the threaded scheduler of Fase 8).
            ("Atomic", "make") => {
                let v = args.first().map(|x| num_disp(x)).unwrap_or(0);
                Ok(Val::Int(atomic_new(v)))
            }
            ("Atomic", "get") => {
                let id = args.first().map(|x| num_disp(x)).unwrap_or(0);
                Ok(Val::Int(atomic_load(id)))
            }
            ("Atomic", "set") => {
                let id = args.first().map(|x| num_disp(x)).unwrap_or(0);
                let v = args.get(1).map(|x| num_disp(x)).unwrap_or(0);
                atomic_store(id, v);
                Ok(Val::Unit)
            }
            ("Atomic", "add") => {
                let id = args.first().map(|x| num_disp(x)).unwrap_or(0);
                let d = args.get(1).map(|x| num_disp(x)).unwrap_or(0);
                Ok(Val::Int(atomic_add(id, d)))
            }
            ("Atomic", "swap") => {
                let id = args.first().map(|x| num_disp(x)).unwrap_or(0);
                let v = args.get(1).map(|x| num_disp(x)).unwrap_or(0);
                Ok(Val::Int(atomic_swap(id, v)))
            }
            ("Atomic", "cas") => {
                let id = args.first().map(|x| num_disp(x)).unwrap_or(0);
                let old = args.get(1).map(|x| num_disp(x)).unwrap_or(0);
                let new = args.get(2).map(|x| num_disp(x)).unwrap_or(0);
                Ok(Val::Bool(atomic_cas(id, old, new)))
            }
            ("Atomic", "drop") => {
                let id = args.first().map(|x| num_disp(x)).unwrap_or(0);
                atomic_free(id);
                Ok(Val::Unit)
            }
            // Rune/code-point access (backs `unicode/utf8` and `bytes`).
            ("Text", "code_at") => {
                let s = args.first().map(disp).unwrap_or_default();
                let i = args.get(1).map(|v| v.display().parse::<i64>().unwrap_or(0)).unwrap_or(0);
                Ok(Val::Int(text_code_at(&s, i)))
            }
            ("Text", "from_code") => {
                let n = args.first().map(|v| v.display().parse::<u32>().unwrap_or(0)).unwrap_or(0);
                Ok(Val::Str(
                    char::from_u32(n).map(|c| c.to_string()).unwrap_or_default(),
                ))
            }
            ("Text", "len") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Int(text_len(&s)))
            }
            // Fast substring/search primitives (back `strings`, `bytes`,
            // `path` — O(n) instead of O(n^2) `char_at` loops in Lex).
            ("Text", "slice") => {
                let s = args.first().map(disp).unwrap_or_default();
                let begin = args.get(1).map(|v| num_disp(v)).unwrap_or(0);
                let end = args.get(2).map(|v| num_disp(v)).unwrap_or(0);
                Ok(Val::Str(text_slice(&s, begin, end)))
            }
            ("Text", "index_of") => {
                let s = args.first().map(disp).unwrap_or_default();
                let sub = args.get(1).map(disp).unwrap_or_default();
                Ok(Val::Int(text_index(&s, &sub)))
            }
            ("Text", "last_index_of") => {
                let s = args.first().map(disp).unwrap_or_default();
                let sub = args.get(1).map(disp).unwrap_or_default();
                Ok(Val::Int(text_last_index(&s, &sub)))
            }
            ("Text", "starts_with") => {
                let s = args.first().map(disp).unwrap_or_default();
                let sub = args.get(1).map(disp).unwrap_or_default();
                Ok(Val::Bool(s.starts_with(sub.as_str())))
            }
            ("Text", "ends_with") => {
                let s = args.first().map(disp).unwrap_or_default();
                let sub = args.get(1).map(disp).unwrap_or_default();
                Ok(Val::Bool(s.ends_with(sub.as_str())))
            }
            ("Text", "repeat") => {
                let s = args.first().map(disp).unwrap_or_default();
                let n = args.get(1).map(|v| num_disp(v)).unwrap_or(0);
                if n <= 0 {
                    return Ok(Val::Str(String::new()));
                }
                if s.len() as i64 * n > 1_000_000 {
                    return runtime("Text::repeat result too large");
                }
                Ok(Val::Str(s.repeat(n as usize)))
            }
            ("Text", "join") => {
                let sep = args.get(1).map(disp).unwrap_or_default();
                match args.first() {
                    Some(Val::List(items)) => {
                        let parts: Vec<String> =
                            items.iter().map(|v| v.display()).collect();
                        Ok(Val::Str(parts.join(sep.as_str())))
                    }
                    Some(other) => runtime(format!(
                        "Text::join expects a list, found {}",
                        other.type_name()
                    )),
                    None => Ok(Val::Str(String::new())),
                }
            }
            // Native float64 math (backs `math`: exact + ~100x faster than
            // the pure-Lex series; the `.lex` layer keeps Go's API names).
            ("Math", "sin") => Ok(Val::Float(farg(&args, 0)? .sin())),
            ("Math", "cos") => Ok(Val::Float(farg(&args, 0)? .cos())),
            ("Math", "tan") => Ok(Val::Float(farg(&args, 0)? .tan())),
            ("Math", "asin") => Ok(Val::Float(farg(&args, 0)? .asin())),
            ("Math", "acos") => Ok(Val::Float(farg(&args, 0)? .acos())),
            ("Math", "atan") => Ok(Val::Float(farg(&args, 0)? .atan())),
            ("Math", "atan2") => Ok(Val::Float(farg(&args, 0)?.atan2(farg(&args, 1)?))),
            ("Math", "sqrt") => Ok(Val::Float(farg(&args, 0)? .sqrt())),
            ("Math", "cbrt") => Ok(Val::Float(farg(&args, 0)? .cbrt())),
            ("Math", "exp") => Ok(Val::Float(farg(&args, 0)? .exp())),
            ("Math", "ln") => Ok(Val::Float(farg(&args, 0)? .ln())),
            ("Math", "log10") => Ok(Val::Float(farg(&args, 0)? .log10())),
            ("Math", "log2") => Ok(Val::Float(farg(&args, 0)? .log2())),
            ("Math", "log") => {
                let x = farg(&args, 0)?;
                let b = farg(&args, 1)?;
                Ok(Val::Float(x.log(b)))
            }
            ("Math", "pow") => Ok(Val::Float(farg(&args, 0)?.powf(farg(&args, 1)?))),
            ("Math", "hypot") => Ok(Val::Float(farg(&args, 0)?.hypot(farg(&args, 1)?))),
            ("Math", "floor") => Ok(Val::Float(farg(&args, 0)? .floor())),
            ("Math", "ceil") => Ok(Val::Float(farg(&args, 0)? .ceil())),
            ("Math", "round") => Ok(Val::Float(farg(&args, 0)? .round())),
            ("Math", "trunc") => Ok(Val::Float(farg(&args, 0)? .trunc())),
            ("Math", "abs") => Ok(Val::Float(farg(&args, 0)? .abs())),
            ("Math", "sign") => {
                let x = farg(&args, 0)?;
                Ok(Val::Float(if x < 0.0 { -1.0 } else if x > 0.0 { 1.0 } else { 0.0 }))
            }
            ("Math", "fmod") => {
                let x = farg(&args, 0)?;
                let y = farg(&args, 1)?;
                if y == 0.0 {
                    return runtime("Math::fmod by zero");
                }
                Ok(Val::Float(x % y))
            }
            ("Math", "max") => match (&args.first(), &args.get(1)) {
                (Some(Val::Int(a)), Some(Val::Int(b))) => Ok(Val::Int(*a.max(b))),
                _ => Ok(Val::Float(farg(&args, 0)?.max(farg(&args, 1)?))),
            },
            ("Math", "min") => match (&args.first(), &args.get(1)) {
                (Some(Val::Int(a)), Some(Val::Int(b))) => Ok(Val::Int(*a.min(b))),
                _ => Ok(Val::Float(farg(&args, 0)?.min(farg(&args, 1)?))),
            },
            // Native sorts (backs `sort`: Rust `sort_unstable`, no extra
            // allocations per level like the Lex merge sort).
            ("List", "sort_ints") => {
                let mut nums = match args.first() {
                    Some(Val::List(items)) => {
                        let mut out = Vec::with_capacity(items.len());
                        for v in items {
                            match v {
                                Val::Int(n) => out.push(*n),
                                other => {
                                    return runtime(format!(
                                        "List::sort_ints expects ints, found {}",
                                        other.type_name()
                                    ))
                                }
                            }
                        }
                        out
                    }
                    Some(other) => {
                        return runtime(format!(
                            "List::sort_ints expects a list, found {}",
                            other.type_name()
                        ))
                    }
                    None => Vec::new(),
                };
                nums.sort_unstable();
                Ok(Val::List(nums.into_iter().map(Val::Int).collect()))
            }
            ("List", "sort_floats") => {
                let mut nums = match args.first() {
                    Some(Val::List(items)) => {
                        let mut out = Vec::with_capacity(items.len());
                        for v in items {
                            match v {
                                Val::Float(x) => out.push(*x),
                                Val::Int(n) => out.push(*n as f64),
                                other => {
                                    return runtime(format!(
                                        "List::sort_floats expects numbers, found {}",
                                        other.type_name()
                                    ))
                                }
                            }
                        }
                        out
                    }
                    Some(other) => {
                        return runtime(format!(
                            "List::sort_floats expects a list, found {}",
                            other.type_name()
                        ))
                    }
                    None => Vec::new(),
                };
                nums.sort_by(|a, b| a.total_cmp(b));
                Ok(Val::List(nums.into_iter().map(Val::Float).collect()))
            }
            ("List", "sort_strings") => {
                let mut strs = match args.first() {
                    Some(Val::List(items)) => {
                        let mut out = Vec::with_capacity(items.len());
                        for v in items {
                            match v {
                                Val::Str(s) => out.push(s.clone()),
                                other => {
                                    return runtime(format!(
                                        "List::sort_strings expects strings, found {}",
                                        other.type_name()
                                    ))
                                }
                            }
                        }
                        out
                    }
                    Some(other) => {
                        return runtime(format!(
                            "List::sort_strings expects a list, found {}",
                            other.type_name()
                        ))
                    }
                    None => Vec::new(),
                };
                strs.sort();
                Ok(Val::List(strs.into_iter().map(Val::Str).collect()))
            }
            ("List", "reverse") => match args.first() {
                Some(Val::List(items)) => {
                    let mut out = items.clone();
                    out.reverse();
                    Ok(Val::List(out))
                }
                Some(other) => runtime(format!(
                    "List::reverse expects a list, found {}",
                    other.type_name()
                )),
                None => Ok(Val::List(Vec::new())),
            },
            // Hash primitives (backs `hash/*`: allocation-free, one pass).
            ("Hash", "fnv1a32") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Int(fnv1a32(s.as_bytes()) as i64))
            }
            ("Hash", "fnv1a64") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Int(fnv1a64(s.as_bytes()) as i64))
            }
            ("Hash", "crc32") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Int(crc32_ieee(s.as_bytes()) as i64))
            }
            ("Hash", "adler32") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Int(adler32(s.as_bytes()) as i64))
            }
            ("Hash", "crc64") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Int(crc64_ecma(s.as_bytes()) as i64))
            }
            ("Hash", "maphash") => {
                let seed = args.first().map(|v| num_disp(v)).unwrap_or(0);
                let s = args.get(1).map(disp).unwrap_or_default();
                Ok(Val::Int(maphash64(seed as u64, s.as_bytes()) as i64))
            }
            ("Hash", "sha256hex") => {
                let s = args.first().map(disp).unwrap_or_default();
                Ok(Val::Str(sha256_hex(s.as_bytes())))
            }
            // Host facts (backs `runtime`: real values, no guessing).
            ("Sys", "goos") => Ok(Val::Str(std::env::consts::OS.to_string())),
            ("Sys", "arch") => Ok(Val::Str(std::env::consts::ARCH.to_string())),
            ("Sys", "ncpu") => Ok(Val::Int(
                std::thread::available_parallelism().map(|n| n.get() as i64).unwrap_or(1),
            )),
            // Toolchain version baked at compile time (backs
            // `runtime::Version`; always tracks release auto-bumps).
            ("Sys", "lex_version") => Ok(Val::Str(env!("CARGO_PKG_VERSION").to_string())),
            // PRNG (backs `math/rand`: xorshift64*, seeded from wall clock).
            ("Rand", "intn") => {
                let n = args.first().map(|v| num_disp(v)).unwrap_or(0);
                if n <= 0 {
                    return runtime("Rand::intn needs n > 0");
                }
                Ok(Val::Int((rand_u64() % (n as u64)) as i64))
            }
            ("Rand", "int") => Ok(Val::Int(rand_u64() as i64)),
            ("Rand", "float") => {
                Ok(Val::Float((rand_u64() as f64) / (u64::MAX as f64)))
            }
            ("Rand", "bytes") => {
                let n = args.first().map(|v| num_disp(v)).unwrap_or(0);
                if n < 0 || n > 1_000_000 {
                    return runtime("Rand::bytes needs 0 <= n <= 1000000");
                }
                let mut out = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    out.push(Val::Int((rand_u64() % 256) as i64));
                }
                Ok(Val::List(out))
            }
            ("Rand", "seed") => {
                let s = args.first().map(|v| num_disp(v)).unwrap_or(0);
                rand_seed(s as u64 | 1);
                Ok(Val::Unit)
            }
            ("Http", "serve") => Ok(Val::Unit), // booted separately by `lex run`
            ("Http", "get") | ("Http", "post") | ("Http", "put") | ("Http", "delete") | ("Http", "head") => {
                let method = member.to_uppercase();
                let url = args.first().map(disp).unwrap_or_default();
                if url.is_empty() || !url.starts_with("http") {
                    return Ok(Val::Str(String::new()));
                }
                let body = if method == "POST" || method == "PUT" {
                    args.get(1).map(disp).filter(|s| !s.is_empty())
                } else {
                    None
                };
                let body = crate::compiler::do_mock_http(
                    &method,
                    &url,
                    body.as_deref(),
                    &[],
                );
                Ok(Val::Str(body))
            }
            _ => unsupported(format!("`{}::{}`", module, member)),
        }
    }

    fn eval_method(&mut self, object: &Expr, method: &str, args: &[Expr]) -> IResult<Val> {
        // `Module.member(...)` in DOT form (`Console.writeLine`, `Http.get`,
        // `Json.serialize`, ...) parses as a method call, not as
        // `Call(FieldAccess)`. When the receiver is an UNBOUND PascalCase
        // identifier it names a module — delegate to the module builtins.
        // (Bound lowercase variables keep normal method semantics.)
        if let Expr::Ident(id) = object {
            let starts_upper = id.text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            if starts_upper && self.lookup(&id.text).is_none() {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a)?);
                }
                return self.call_module(&id.text, method, vals);
            }
        }
        // Handles do motor gráfico (`window.setSize(...)`, `bar.add(...)`,
        // `nome.setPlaceholder(...)`, ...): structs com campo `id` de tipos
        // gráficos conhecidos.
        #[cfg(feature = "gui")]
        if let Expr::Ident(id) = object {
            if let Some(Val::Struct { name, fields }) = self.lookup(&id.text) {
                if matches!(
                    name.as_str(),
                    "Window" | "MenuBar"
                        | "Menu"
                        | "Label"
                        | "Button"
                        | "TextField"
                        | "Checkbox"
                        | "MenuItem"
                ) {
                    if let Some((_, Val::Int(h))) = fields.iter().find(|(k, _)| k == "id") {
                        let mut vals = Vec::with_capacity(args.len());
                        for a in args {
                            vals.push(self.eval(a)?);
                        }
                        return self.gfx_method(&name, *h, method, &vals);
                    }
                }
            }
        }
        // `value.method(args)` where the struct holds a function field
        // under that name (same case as `Call(FieldAccess)` above, but in
        // method-call syntax: `h.less(a, b)`).
        if let Expr::Ident(id) = object {
            if let Some(Val::Struct { fields, .. }) = self.lookup(&id.text) {
                if let Some((_, Val::Func(g))) =
                    fields.iter().find(|(k, _)| k == method)
                {
                    let g = g.clone();
                    let mut vals = Vec::with_capacity(args.len());
                    for a in args {
                        vals.push(self.eval(a)?);
                    }
                    return self.call_func(&g, vals);
                }
            }
        }
        match method {
            "len" | "length" | "count" | "size" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                match self.eval(object)? {
                    Val::List(items) => Ok(Val::Int(items.len() as i64)),
                    Val::Tuple(items) => Ok(Val::Int(items.len() as i64)),
                    Val::Str(s) => Ok(Val::Int(text_len(&s))),
                    Val::Struct { fields, .. } => Ok(Val::Int(fields.len() as i64)),
                    other => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "isEmpty" | "is_empty" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                match self.eval(object)? {
                    Val::List(items) => Ok(Val::Bool(items.is_empty())),
                    Val::Tuple(items) => Ok(Val::Bool(items.is_empty())),
                    Val::Str(s) => Ok(Val::Bool(s.is_empty())),
                    other => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "push" => {
                if args.len() != 1 {
                    return runtime("`push` takes exactly one argument");
                }
                let v = self.eval(&args[0])?;
                match object {
                    Expr::Ident(id) => {
                        let mut obj = self.lookup(&id.text).ok_or_else(|| {
                            InterpError::Runtime(format!("undefined variable `{}`", id.text))
                        })?;
                        match &mut obj {
                            Val::List(items) => {
                                items.push(v);
                                self.assign(&id.text, obj);
                                Ok(Val::Unit)
                            }
                            other => runtime(format!("cannot `push` onto {}", other.type_name())),
                        }
                    }
                    _ => unsupported("`push` on complex receiver"),
                }
            }
            "pop" => {
                if !args.is_empty() {
                    return runtime("`pop` takes no arguments");
                }
                match object {
                    Expr::Ident(id) => {
                        let mut obj = self.lookup(&id.text).ok_or_else(|| {
                            InterpError::Runtime(format!("undefined variable `{}`", id.text))
                        })?;
                        match &mut obj {
                            Val::List(items) => {
                                let v = items.pop().unwrap_or(Val::Null);
                                self.assign(&id.text, obj);
                                Ok(v)
                            }
                            other => runtime(format!("cannot `pop` from {}", other.type_name())),
                        }
                    }
                    _ => unsupported("`pop` on complex receiver"),
                }
            }
            "contains" => {
                if args.len() != 1 {
                    return runtime("`contains` takes exactly one argument");
                }
                let needle = self.eval(&args[0])?;
                match self.eval(object)? {
                    Val::List(items) => Ok(Val::Bool(items.iter().any(|v| v.equals(&needle)))),
                    Val::Str(s) => Ok(Val::Bool(s.contains(&needle.display()))),
                    other => runtime(format!("`contains` not supported on {}", other.type_name())),
                }
            }
            "to_string" | "toString" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                Ok(Val::Str(self.eval(object)?.display()))
            }
            // Runtime reflection in method position (`x.type()`, promised
            // next to the `type(x)` callee alias — same `type_of` answer).
            "type" | "typeof" | "typeOf" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                Ok(Val::Str(self.eval(object)?.type_of()))
            }
            // String primitives used by the Lex stdlib (`lib/std/strings.lex`).
            "char_at" | "charAt" => {
                if args.len() != 1 {
                    return runtime(format!("`{}` takes one index argument", method));
                }
                let idx = self.eval(&args[0])?;
                match (self.eval(object)?, idx) {
                    (Val::Str(s), Val::Int(i)) => {
                        let n = text_len(&s);
                        if i < 0 || i >= n {
                            return runtime(format!("char index {} out of bounds (len {})", i, n));
                        }
                        if s.is_ascii() {
                            Ok(Val::Char(s.as_bytes()[i as usize] as char))
                        } else {
                            Ok(Val::Char(s.chars().nth(i as usize).unwrap_or('\0')))
                        }
                    }
                    (other, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "to_upper" | "toUpper" | "uppercase" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                match self.eval(object)? {
                    Val::Str(s) => Ok(Val::Str(s.to_uppercase())),
                    other => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "to_lower" | "toLower" | "lowercase" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                match self.eval(object)? {
                    Val::Str(s) => Ok(Val::Str(s.to_lowercase())),
                    other => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "trim" | "trim_space" | "trimSpace" => {
                if !args.is_empty() {
                    return runtime(format!("`{}` takes no arguments", method));
                }
                match self.eval(object)? {
                    Val::Str(s) => Ok(Val::Str(s.trim().to_string())),
                    other => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "split" => {
                if args.len() != 1 {
                    return runtime(format!("`{}` takes one separator argument", method));
                }
                let sep = self.eval(&args[0])?;
                match (self.eval(object)?, sep) {
                    (Val::Str(s), Val::Str(delim)) => {
                        let parts: Vec<Val> = if delim.is_empty() {
                            s.chars().map(|c| Val::Str(c.to_string())).collect()
                        } else {
                            s.split(&delim as &str).map(|p| Val::Str(p.to_string())).collect()
                        };
                        Ok(Val::List(parts))
                    }
                    (other, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "replace" | "replace_all" | "replaceAll" => {
                if args.len() != 2 {
                    return runtime(format!("`{}` takes (from, to) arguments", method));
                }
                let from = self.eval(&args[0])?;
                let to = self.eval(&args[1])?;
                match (self.eval(object)?, from, to) {
                    (Val::Str(s), Val::Str(a), Val::Str(b)) => Ok(Val::Str(s.replace(&a, &b))),
                    (other, _, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "starts_with" | "startsWith" | "has_prefix" | "hasPrefix" => {
                if args.len() != 1 {
                    return runtime(format!("`{}` takes one prefix argument", method));
                }
                let prefix = self.eval(&args[0])?;
                match (self.eval(object)?, prefix) {
                    (Val::Str(s), Val::Str(p)) => Ok(Val::Bool(s.starts_with(p.as_str()))),
                    (other, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "ends_with" | "endsWith" | "has_suffix" | "hasSuffix" => {
                if args.len() != 1 {
                    return runtime(format!("`{}` takes one suffix argument", method));
                }
                let suffix = self.eval(&args[0])?;
                match (self.eval(object)?, suffix) {
                    (Val::Str(s), Val::Str(p)) => Ok(Val::Bool(s.ends_with(p.as_str()))),
                    (other, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "index_of" | "indexOf" => {
                if args.len() != 1 {
                    return runtime(format!("`{}` takes one substring argument", method));
                }
                let sub = self.eval(&args[0])?;
                match (self.eval(object)?, sub) {
                    (Val::Str(s), Val::Str(p)) => Ok(Val::Int(text_index(&s, &p))),
                    (other, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "substring" | "slice" => {
                if args.len() != 2 {
                    return runtime(format!("`{}` takes (begin, end) arguments", method));
                }
                let begin = self.eval(&args[0])?;
                let end = self.eval(&args[1])?;
                match (self.eval(object)?, begin, end) {
                    (Val::Str(s), Val::Int(b), Val::Int(e)) => Ok(Val::Str(text_slice(&s, b, e))),
                    (other, _, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            "repeat" => {
                if args.len() != 1 {
                    return runtime(format!("`{}` takes one count argument", method));
                }
                let n = self.eval(&args[0])?;
                match (self.eval(object)?, n) {
                    (Val::Str(s), Val::Int(k)) => {
                        if k <= 0 {
                            return Ok(Val::Str(String::new()));
                        }
                        if s.len() as i64 * k > 1_000_000 {
                            return runtime("repeat result too large");
                        }
                        Ok(Val::Str(s.repeat(k as usize)))
                    }
                    (other, _) => runtime(format!("`{}` not supported on {}", method, other.type_name())),
                }
            }
            _ => unsupported(format!("method `.{}()`", method)),
        }
    }
}

fn cast_val(v: Val, ty: &lexicon_parser::Type) -> Val {
    // Target type name: primitives map by variant, paths by last segment.
    let name = match ty {
        lexicon_parser::Type::Primitive(p) => format!("{:?}", p).to_lowercase(),
        lexicon_parser::Type::Path(p) => {
            p.segments.last().map(|s| s.text.clone()).unwrap_or_default()
        }
        _ => String::new(),
    }
    .to_lowercase();
    let disp = v.display();
    match name.as_str() {
        "string" => Val::Str(disp),
        "bool" => Val::Bool(matches!(v, Val::Bool(true)) || disp == "true" || disp == "1"),
        "char" => Val::Char(disp.chars().next().unwrap_or('\0')),
        "f32" | "f64" | "float" | "float32" | "float64" | "decimal" => Val::Float(
            disp.parse::<f64>().unwrap_or_else(|_| match v {
                Val::Int(n) => n as f64,
                Val::Bool(b) => {
                    if b {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }),
        ),
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "int"
        | "uint" | "byte" => Val::Int(disp.parse::<i64>().unwrap_or_else(|_| match v {
            Val::Float(x) => x as i64,
            Val::Bool(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        })),
        // Unknown target: keep the value (lenient — never fail a cast).
        _ => v,
    }
}

fn apply_arith(op: BinOp, l: Val, r: Val) -> IResult<Val> {
    // String concatenation with `+` (either side a string).
    if op == BinOp::Add {
        match (&l, &r) {
            (Val::Str(a), b) => return Ok(Val::Str(format!("{}{}", a, b.display()))),
            (a, Val::Str(b)) => return Ok(Val::Str(format!("{}{}", a.display(), b))),
            _ => {}
        }
    }
    // Exact i64 fast path: integers never round-trip through f64, so
    // bit operations and wrapping arithmetic stay bit-exact past 2^53
    // (this is what keeps `hash/*` and crypto code correct) and skip the
    // float conversion entirely.
    match (l, r) {
        (Val::Int(a), Val::Int(b)) => {
            let shift = |f: &dyn Fn(i64, i64) -> Option<i64>| -> IResult<Val> {
                f(a, b)
                    .map(Val::Int)
                    .ok_or_else(|| InterpError::Runtime("arithmetic overflow".to_string()))
            };
            match op {
                BinOp::Add => Ok(Val::Int(a.wrapping_add(b))),
                BinOp::Sub => Ok(Val::Int(a.wrapping_sub(b))),
                BinOp::Mul => Ok(Val::Int(a.wrapping_mul(b))),
                BinOp::Div => {
                    if b == 0 {
                        return runtime("division by zero");
                    }
                    Ok(Val::Int(a.wrapping_div(b)))
                }
                BinOp::Mod => {
                    if b == 0 {
                        return runtime("modulo by zero");
                    }
                    Ok(Val::Int(a.wrapping_rem(b)))
                }
                BinOp::Shl => shift(&|x, y| x.checked_shl(y as u32)),
                BinOp::Shr => shift(&|x, y| x.checked_shr(y as u32)),
                BinOp::BitAnd => Ok(Val::Int(a & b)),
                BinOp::BitOr => Ok(Val::Int(a | b)),
                BinOp::BitXor => Ok(Val::Int(a ^ b)),
                _ => unsupported("binary operator"),
            }
        }
        (Val::Float(a), Val::Float(b)) => apply_float_arith(op, a, b),
        (Val::Int(a), Val::Float(b)) => apply_float_arith(op, a as f64, b),
        (Val::Float(a), Val::Int(b)) => apply_float_arith(op, a, b as f64),
        (l, r) => runtime(format!(
            "arithmetic on non-numbers ({} and {})",
            l.type_name(),
            r.type_name()
        )),
    }
}

fn apply_float_arith(op: BinOp, a: f64, b: f64) -> IResult<Val> {
    match op {
        BinOp::Add => Ok(Val::Float(a + b)),
        BinOp::Sub => Ok(Val::Float(a - b)),
        BinOp::Mul => Ok(Val::Float(a * b)),
        BinOp::Div => {
            if b == 0.0 {
                return runtime("division by zero");
            }
            Ok(Val::Float(a / b))
        }
        BinOp::Mod => {
            if b == 0.0 {
                return runtime("modulo by zero");
            }
            Ok(Val::Float(a % b))
        }
        BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
            runtime("integer-only operation on floats")
        }
        _ => unsupported("binary operator"),
    }
}

fn compare(op: BinOp, l: &Val, r: &Val) -> IResult<Val> {
    // Exact integer comparison first (no f64 round-trip past 2^53),
    // then mixed numeric comparison, then lexicographic strings.
    if let (Val::Int(a), Val::Int(b)) = (l, r) {
        return Ok(Val::Bool(match op {
            BinOp::Lt => a < b,
            BinOp::LtEq => a <= b,
            BinOp::Gt => a > b,
            BinOp::GtEq => a >= b,
            _ => return unsupported("comparison operator"),
        }));
    }
    let nums = match (l, r) {
        (Val::Float(a), Val::Float(b)) => Some((*a, *b)),
        (Val::Int(a), Val::Float(b)) => Some((*a as f64, *b)),
        (Val::Float(a), Val::Int(b)) => Some((*a, *b as f64)),
        _ => None,
    };
    if let Some((a, b)) = nums {
        return Ok(Val::Bool(match op {
            BinOp::Lt => a < b,
            BinOp::LtEq => a <= b,
            BinOp::Gt => a > b,
            BinOp::GtEq => a >= b,
            _ => return unsupported("comparison operator"),
        }));
    }
    // Lexicographic string comparison.
    if let (Val::Str(a), Val::Str(b)) = (l, r) {
        return Ok(Val::Bool(match op {
            BinOp::Lt => a < b,
            BinOp::LtEq => a <= b,
            BinOp::Gt => a > b,
            BinOp::GtEq => a >= b,
            _ => return unsupported("comparison operator"),
        }));
    }
    runtime(format!(
        "cannot compare {} and {}",
        l.type_name(),
        r.type_name()
    ))
}

// ---------------------------------------------------------------------------
// Native speed helpers (back the Lex stdlib — one pass, no interpreter
// overhead per character; this is what makes tight loops competitive).
// ---------------------------------------------------------------------------

/// Numeric view of an argument (ints, floats, numeric strings).
fn num_disp(v: &Val) -> i64 {
    match v {
        Val::Int(n) => *n,
        Val::Float(x) => *x as i64,
        Val::Bool(b) => {
            if *b {
                1
            } else {
                0
            }
        }
        other => other.display().parse::<i64>().unwrap_or(0),
    }
}

/// Float view of the i-th call argument (missing/non-numeric = error).
fn farg(args: &[Val], i: usize) -> IResult<f64> {
    match args.get(i) {
        Some(Val::Int(n)) => Ok(*n as f64),
        Some(Val::Float(x)) => Ok(*x),
        Some(other) => other
            .display()
            .parse::<f64>()
            .map_err(|_| InterpError::Runtime(format!("expected number, found {}", other.type_name()))),
        None => runtime("missing numeric argument"),
    }
}

/// Char-offset slice `[begin:end)` in O(n) (clamped, never panics).
/// ASCII fast path: byte indexing, no allocation or decode walk.
fn text_slice(s: &str, begin: i64, end: i64) -> String {
    if s.is_ascii() {
        let n = s.len() as i64;
        let mut b = begin.max(0).min(n);
        let mut e = end.max(0).min(n);
        if b > e {
            std::mem::swap(&mut b, &mut e);
        }
        return s[b as usize..e as usize].to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len() as i64;
    let mut b = begin.max(0).min(n);
    let mut e = end.max(0).min(n);
    if b > e {
        std::mem::swap(&mut b, &mut e);
    }
    chars[b as usize..e as usize].iter().collect()
}

/// Char count with an ASCII fast path (byte length, no decode walk).
fn text_len(s: &str) -> i64 {
    if s.is_ascii() {
        return s.len() as i64;
    }
    s.chars().count() as i64
}

/// Code point at a char offset (-1 when out of range), ASCII fast path.
fn text_code_at(s: &str, i: i64) -> i64 {
    if i < 0 {
        return -1;
    }
    if s.is_ascii() {
        return s.as_bytes().get(i as usize).map(|b| *b as i64).unwrap_or(-1);
    }
    s.chars().nth(i as usize).map(|c| c as i64).unwrap_or(-1)
}

/// Char-offset `indexOf` (Go `strings.Index` semantics, -1 when absent).
fn text_index(s: &str, sub: &str) -> i64 {
    if sub.is_empty() {
        return 0;
    }
    match s.find(sub) {
        Some(byte) => s[..byte].chars().count() as i64,
        None => -1,
    }
}

fn text_last_index(s: &str, sub: &str) -> i64 {
    if sub.is_empty() {
        return s.chars().count() as i64;
    }
    match s.rfind(sub) {
        Some(byte) => s[..byte].chars().count() as i64,
        None => -1,
    }
}

fn fnv1a32(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in data {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

fn crc32_ieee(data: &[u8]) -> u32 {
    // Bit-by-bit IEEE CRC-32 (no 1 KB table — keeps the binary small;
    // still ~50x faster than a per-bit loop inside the interpreter).
    let mut crc: u32 = 0xffff_ffff;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xedb8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for byte in data {
        a = (a + *byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn crc64_ecma(data: &[u8]) -> u64 {
    // Bit-by-bit ECMA-182 (poly 0xC96C5795D7870F42, reflected) — same
    // no-table policy as `crc32_ieee`, still far faster than a Lex loop.
    let mut crc: u64 = 0xffff_ffff_ffff_ffff;
    for b in data {
        crc ^= *b as u64;
        for _ in 0..8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xc96c_5795_d787_0f42;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn maphash64(seed: u64, data: &[u8]) -> u64 {    // Seeded FNV-1a64 (Go `maphash` shape: per-map random seed mixed in;
    // the seed here travels explicitly since hashes cross as values).
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// FIPS 180-4 SHA-256, hex digest (backs `sha256::Sum`: the pure-Lex
/// compression stays as the documented reference + streaming path, while
/// the one-shot delegates here — same split as `sort::Ints` over
/// `List::sort_ints`).
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
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
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bitlen = (msg.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([block[4 * i], block[4 * i + 1], block[4 * i + 2], block[4 * i + 3]]);
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
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
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
    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{:08x}", word));
    }
    out
}
static ATOMIC_NEXT: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

/// Script arguments for the running program (`lex run prog.lex -- args…`).
/// Mirrors Go's `os.Args`: `[program, user args…]`. Set once per `run`;
/// when unset, `Process::args` falls back to the toolchain's own argv.
static SCRIPT_ARGS: std::sync::OnceLock<std::sync::Mutex<Vec<String>>> =
    std::sync::OnceLock::new();

pub(crate) fn set_script_args(args: Vec<String>) {
    let slot = SCRIPT_ARGS.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    if let Ok(mut guard) = slot.lock() {
        *guard = args;
    }
}

fn script_args() -> Option<Vec<String>> {
    SCRIPT_ARGS
        .get()
        .and_then(|slot| slot.lock().ok())
        .map(|guard| guard.clone())
        .filter(|v| !v.is_empty())
}

fn atomic_cells() -> &'static std::sync::Mutex<std::collections::HashMap<i64, std::sync::Arc<std::sync::atomic::AtomicI64>>> {
    static CELLS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<i64, std::sync::Arc<std::sync::atomic::AtomicI64>>>,
    > = std::sync::OnceLock::new();
    CELLS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn atomic_new(v: i64) -> i64 {
    use std::sync::atomic::Ordering;
    let id = ATOMIC_NEXT.fetch_add(1, Ordering::Relaxed);
    let cells = atomic_cells();
    if let Ok(mut m) = cells.lock() {
        m.insert(id, std::sync::Arc::new(std::sync::atomic::AtomicI64::new(v)));
    }
    id
}

fn atomic_get(id: i64) -> Option<std::sync::Arc<std::sync::atomic::AtomicI64>> {
    atomic_cells().lock().ok()?.get(&id).cloned()
}

fn atomic_load(id: i64) -> i64 {
    use std::sync::atomic::Ordering;
    atomic_get(id).map(|c| c.load(Ordering::SeqCst)).unwrap_or(0)
}

fn atomic_store(id: i64, v: i64) {
    use std::sync::atomic::Ordering;
    if let Some(c) = atomic_get(id) {
        c.store(v, Ordering::SeqCst);
    }
}

fn atomic_add(id: i64, d: i64) -> i64 {
    use std::sync::atomic::Ordering;
    atomic_get(id).map(|c| c.fetch_add(d, Ordering::SeqCst) + d).unwrap_or(d)
}

fn atomic_swap(id: i64, v: i64) -> i64 {
    use std::sync::atomic::Ordering;
    atomic_get(id).map(|c| c.swap(v, Ordering::SeqCst)).unwrap_or(0)
}

fn atomic_cas(id: i64, old: i64, new: i64) -> bool {
    use std::sync::atomic::Ordering;
    atomic_get(id)
        .map(|c| c.compare_exchange(old, new, Ordering::SeqCst, Ordering::SeqCst).is_ok())
        .unwrap_or(false)
}

fn atomic_free(id: i64) {
    if let Ok(mut m) = atomic_cells().lock() {
        m.remove(&id);
    }
}
static RAND_STATE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

fn rand_seed(seed: u64) {
    use std::sync::atomic::Ordering;
    let s = if seed == 0 { 0x9e37_79b9_7f4a_7c15 } else { seed };
    RAND_STATE.store(s | 1, Ordering::Relaxed);
}
fn rand_u64() -> u64 {
    use std::sync::atomic::Ordering;
    let mut s = RAND_STATE.load(Ordering::Relaxed);
    if s == 0 {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9e37_79b9_7f4a_7c15)
            | 1;
        RAND_STATE.store(seed, Ordering::Relaxed);
        s = seed;
    }
    // xorshift64* (Vigna): full-period, 3 shifts + 1 multiply.
    s ^= s >> 12;
    s ^= s << 25;
    s ^= s >> 27;
    RAND_STATE.store(s, Ordering::Relaxed);
    s.wrapping_mul(0x2545_f491_4f6c_dd1d)
}
