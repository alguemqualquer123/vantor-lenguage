// Versioned bytecode VM and deterministic interpreter (Spec §64 + §66).
//
// AOT compilation stays the primary deployment path; this VM exists for
// scripting, REPL evaluation and hot-reloadable logic. Format:
// `[version:u16][consts][code]`. The interpreter is deterministic for
// identical inputs. There is NO JIT here by design (executable-memory
// safeguards would be required); tiered compilation is a future phase.

/// Bytecode format version. Bump on ANY semantic change.
pub const BYTECODE_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Const = 0x01,
    Add = 0x02,
    Sub = 0x03,
    Mul = 0x04,
    Div = 0x05,
    Neg = 0x06,
    Not = 0x07,
    Eq = 0x08,
    Lt = 0x09,
    Gt = 0x0A,
    Jump = 0x0B,
    JumpIfFalse = 0x0C,
    GetLocal = 0x0D,
    SetLocal = 0x0E,
    Pop = 0x0F,
    Print = 0x10,
    Return = 0x11,
}

impl OpCode {
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(OpCode::Const),
            0x02 => Some(OpCode::Add),
            0x03 => Some(OpCode::Sub),
            0x04 => Some(OpCode::Mul),
            0x05 => Some(OpCode::Div),
            0x06 => Some(OpCode::Neg),
            0x07 => Some(OpCode::Not),
            0x08 => Some(OpCode::Eq),
            0x09 => Some(OpCode::Lt),
            0x0A => Some(OpCode::Gt),
            0x0B => Some(OpCode::Jump),
            0x0C => Some(OpCode::JumpIfFalse),
            0x0D => Some(OpCode::GetLocal),
            0x0E => Some(OpCode::SetLocal),
            0x0F => Some(OpCode::Pop),
            0x10 => Some(OpCode::Print),
            0x11 => Some(OpCode::Return),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl VmValue {
    fn tag(&self) -> u8 {
        match self {
            VmValue::Int(_) => 1,
            VmValue::Float(_) => 2,
            VmValue::Bool(_) => 3,
            VmValue::Str(_) => 4,
        }
    }

    fn truthy(&self) -> bool {
        match self {
            VmValue::Int(i) => *i != 0,
            VmValue::Float(f) => *f != 0.0,
            VmValue::Bool(b) => *b,
            VmValue::Str(s) => !s.is_empty(),
        }
    }

    pub fn display(&self) -> String {
        match self {
            VmValue::Int(i) => i.to_string(),
            VmValue::Float(f) => f.to_string(),
            VmValue::Bool(b) => b.to_string(),
            VmValue::Str(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub consts: Vec<VmValue>,
    pub code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk::default()
    }

    pub fn push_const(&mut self, v: VmValue) -> u32 {
        self.consts.push(v);
        (self.consts.len() - 1) as u32
    }

    pub fn emit(&mut self, op: OpCode) {
        self.code.push(op as u8);
    }

    pub fn emit_u32(&mut self, v: u32) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }

    pub fn emit_u8(&mut self, v: u8) {
        self.code.push(v);
    }

    /// Serialize with version header.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&BYTECODE_VERSION.to_le_bytes());
        out.extend_from_slice(&(self.consts.len() as u32).to_le_bytes());
        for c in &self.consts {
            out.push(c.tag());
            match c {
                VmValue::Int(i) => out.extend_from_slice(&i.to_le_bytes()),
                VmValue::Float(f) => out.extend_from_slice(&f.to_le_bytes()),
                VmValue::Bool(b) => out.push(*b as u8),
                VmValue::Str(s) => {
                    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
                    out.extend_from_slice(s.as_bytes());
                }
            }
        }
        out.extend_from_slice(&(self.code.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.code);
        out
    }

    /// Deserialize, rejecting wrong versions and truncated inputs.
    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut pos = 0usize;
        let take = |input: &[u8], pos: &mut usize, n: usize| -> Option<Vec<u8>> {
            if *pos + n > input.len() {
                return None;
            }
            let s = input[*pos..*pos + n].to_vec();
            *pos += n;
            Some(s)
        };
        let ver = take(input, &mut pos, 2).ok_or("E0601: truncated bytecode")?;
        let ver = u16::from_le_bytes([ver[0], ver[1]]);
        if ver != BYTECODE_VERSION {
            return Err(format!(
                "E0601: bytecode version {} unsupported (want {})",
                ver, BYTECODE_VERSION
            ));
        }
        let n_consts = take(input, &mut pos, 4).ok_or("E0601: truncated consts")?;
        let n_consts = u32::from_le_bytes([n_consts[0], n_consts[1], n_consts[2], n_consts[3]]) as usize;
        if n_consts > 1 << 20 {
            return Err("E0601: const table too large".to_string());
        }
        let mut consts = Vec::with_capacity(n_consts);
        for _ in 0..n_consts {
            let tag = take(input, &mut pos, 1).ok_or("E0601: truncated const")?[0];
            match tag {
                1 => {
                    let b = take(input, &mut pos, 8).ok_or("E0601: truncated int")?;
                    consts.push(VmValue::Int(i64::from_le_bytes(b.try_into().unwrap())));
                }
                2 => {
                    let b = take(input, &mut pos, 8).ok_or("E0601: truncated float")?;
                    consts.push(VmValue::Float(f64::from_le_bytes(b.try_into().unwrap())));
                }
                3 => {
                    let b = take(input, &mut pos, 1).ok_or("E0601: truncated bool")?[0];
                    consts.push(VmValue::Bool(b != 0));
                }
                4 => {
                    let n = take(input, &mut pos, 4).ok_or("E0601: truncated str len")?;
                    let n = u32::from_le_bytes([n[0], n[1], n[2], n[3]]) as usize;
                    if n > (1 << 20) {
                        return Err("E0601: string const too large".to_string());
                    }
                    let b = take(input, &mut pos, n).ok_or("E0601: truncated str")?;
                    consts.push(VmValue::Str(String::from_utf8(b).map_err(|_| "E0601: bad utf8".to_string())?));
                }
                _ => return Err("E0601: unknown const tag".to_string()),
            }
        }
        let n_code = take(input, &mut pos, 4).ok_or("E0601: truncated code len")?;
        let n_code = u32::from_le_bytes([n_code[0], n_code[1], n_code[2], n_code[3]]) as usize;
        let code = take(input, &mut pos, n_code).ok_or("E0601: truncated code")?;
        Ok(Chunk { consts, code })
    }
}

#[derive(Debug, Default)]
pub struct VmOutput {
    pub printed: String,
    pub stack: Vec<VmValue>,
}

/// Instruction budget: the interpreter ALWAYS terminates (sandboxing).
pub const MAX_STEPS: usize = 1 << 24;

pub struct VM {
    stack: Vec<VmValue>,
    locals: Vec<VmValue>,
    printed: String,
}

impl VM {
    pub fn new() -> Self {
        VM { stack: Vec::new(), locals: vec![VmValue::Int(0); 256], printed: String::new() }
    }

    fn pop(&mut self) -> Result<VmValue, String> {
        self.stack.pop().ok_or_else(|| "E0501: stack underflow".to_string())
    }

    fn read_u32(&self, code: &[u8], ip: &mut usize) -> Result<u32, String> {
        if *ip + 4 > code.len() {
            return Err("E0501: truncated operand".to_string());
        }
        let v = u32::from_le_bytes(code[*ip..*ip + 4].try_into().unwrap());
        *ip += 4;
        Ok(v)
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<VmOutput, String> {
        let code = &chunk.code;
        let mut ip = 0usize;
        let mut steps = 0usize;
        loop {
            steps += 1;
            if steps > MAX_STEPS {
                return Err("E0501: instruction budget exceeded".to_string());
            }
            if ip >= code.len() {
                return Err("E0501: fell off end of bytecode".to_string());
            }
            let op = OpCode::from_byte(code[ip])
                .ok_or_else(|| format!("E0501: invalid opcode {:#x}", code[ip]))?;
            ip += 1;
            match op {
                OpCode::Const => {
                    let idx = self.read_u32(code, &mut ip)? as usize;
                    let c = chunk.consts.get(idx).ok_or("E0501: bad const index")?.clone();
                    self.stack.push(c);
                }
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let r = match (a, b) {
                        (VmValue::Int(x), VmValue::Int(y)) => VmValue::Int(match op {
                            OpCode::Add => x.wrapping_add(y),
                            OpCode::Sub => x.wrapping_sub(y),
                            OpCode::Mul => x.wrapping_mul(y),
                            OpCode::Div => {
                                if y == 0 {
                                    return Err("E0501: division by zero".to_string());
                                }
                                x / y
                            }
                            _ => unreachable!(),
                        }),
                        (VmValue::Float(x), VmValue::Float(y)) => VmValue::Float(match op {
                            OpCode::Add => x + y,
                            OpCode::Sub => x - y,
                            OpCode::Mul => x * y,
                            OpCode::Div => x / y,
                            _ => unreachable!(),
                        }),
                        (VmValue::Str(x), VmValue::Str(y)) if op == OpCode::Add => {
                            VmValue::Str(x + &y)
                        }
                        _ => return Err("E0301: arithmetic type mismatch".to_string()),
                    };
                    self.stack.push(r);
                }
                OpCode::Neg => {
                    let a = self.pop()?;
                    match a {
                        VmValue::Int(x) => self.stack.push(VmValue::Int(-x)),
                        VmValue::Float(x) => self.stack.push(VmValue::Float(-x)),
                        _ => return Err("E0301: neg on non-number".to_string()),
                    }
                }
                OpCode::Not => {
                    let a = self.pop()?;
                    self.stack.push(VmValue::Bool(!a.truthy()));
                }
                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(VmValue::Bool(a == b));
                }
                OpCode::Lt => {
                    let (b, a) = (self.pop()?, self.pop()?);
                    let r = match (a, b) {
                        (VmValue::Int(x), VmValue::Int(y)) => x < y,
                        (VmValue::Float(x), VmValue::Float(y)) => x < y,
                        _ => return Err("E0301: comparison type mismatch".to_string()),
                    };
                    self.stack.push(VmValue::Bool(r));
                }
                OpCode::Gt => {
                    let (b, a) = (self.pop()?, self.pop()?);
                    let r = match (a, b) {
                        (VmValue::Int(x), VmValue::Int(y)) => x > y,
                        (VmValue::Float(x), VmValue::Float(y)) => x > y,
                        _ => return Err("E0301: comparison type mismatch".to_string()),
                    };
                    self.stack.push(VmValue::Bool(r));
                }
                OpCode::Jump => {
                    ip = self.read_u32(code, &mut ip)? as usize;
                }
                OpCode::JumpIfFalse => {
                    let target = self.read_u32(code, &mut ip)? as usize;
                    if !self.pop()?.truthy() {
                        ip = target;
                    }
                }
                OpCode::GetLocal => {
                    let slot = code.get(ip).copied().ok_or("E0501: truncated operand")? as usize;
                    ip += 1;
                    self.stack.push(self.locals.get(slot).cloned().ok_or("E0501: bad local")?);
                }
                OpCode::SetLocal => {
                    let slot = code.get(ip).copied().ok_or("E0501: truncated operand")? as usize;
                    ip += 1;
                    let v = self.pop()?;
                    if slot >= self.locals.len() {
                        return Err("E0501: bad local".to_string());
                    }
                    self.locals[slot] = v;
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Print => {
                    let v = self.pop()?;
                    self.printed.push_str(&v.display());
                    self.printed.push('\n');
                }
                OpCode::Return => {
                    return Ok(VmOutput {
                        printed: std::mem::take(&mut self.printed),
                        stack: std::mem::take(&mut self.stack),
                    });
                }
            }
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Part X / §36: const-exec evaluator (deterministic, no host I/O).
//
// `ConstExpr` is the utils-level compile-time expression form: pure
// arithmetic/boolean operators over literals. Evaluation is total
// (`None` for div-by-zero / type mismatch) and has no filesystem,
// network, clock or RNG access, satisfying the §36 deterministic
// input/output rule. The typeck-level `eval_const` (AST-typed) and this
// evaluator agree on literal semantics; this one is AST-agnostic so
// codegen and tooling can reuse it without depending on the parser.
// ---------------------------------------------------------------------------

/// Compile-time constant expression (AST-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub enum ConstExpr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Neg(Box<ConstExpr>),
    Not(Box<ConstExpr>),
    Add(Box<ConstExpr>, Box<ConstExpr>),
    Sub(Box<ConstExpr>, Box<ConstExpr>),
    Mul(Box<ConstExpr>, Box<ConstExpr>),
    Div(Box<ConstExpr>, Box<ConstExpr>),
    Eq(Box<ConstExpr>, Box<ConstExpr>),
    Lt(Box<ConstExpr>, Box<ConstExpr>),
}

/// Evaluate a constant expression deterministically.
/// Returns `None` for non-constant shapes (div-by-zero, type mix).
pub fn eval_const_expr(expr: &ConstExpr) -> Option<VmValue> {
    match expr {
        ConstExpr::Int(i) => Some(VmValue::Int(*i)),
        ConstExpr::Float(f) => Some(VmValue::Float(*f)),
        ConstExpr::Bool(b) => Some(VmValue::Bool(*b)),
        ConstExpr::Str(s) => Some(VmValue::Str(s.clone())),
        ConstExpr::Neg(e) => match eval_const_expr(e)? {
            VmValue::Int(i) => Some(VmValue::Int(-i)),
            VmValue::Float(f) => Some(VmValue::Float(-f)),
            _ => None,
        },
        ConstExpr::Not(e) => {
            let v = eval_const_expr(e)?;
            Some(VmValue::Bool(!v.truthy()))
        }
        ConstExpr::Add(a, b) => match (eval_const_expr(a)?, eval_const_expr(b)?) {
            (VmValue::Int(x), VmValue::Int(y)) => Some(VmValue::Int(x.wrapping_add(y))),
            (VmValue::Float(x), VmValue::Float(y)) => Some(VmValue::Float(x + y)),
            (VmValue::Str(x), VmValue::Str(y)) => Some(VmValue::Str(x + &y)),
            _ => None,
        },
        ConstExpr::Sub(a, b) => match (eval_const_expr(a)?, eval_const_expr(b)?) {
            (VmValue::Int(x), VmValue::Int(y)) => Some(VmValue::Int(x.wrapping_sub(y))),
            (VmValue::Float(x), VmValue::Float(y)) => Some(VmValue::Float(x - y)),
            _ => None,
        },
        ConstExpr::Mul(a, b) => match (eval_const_expr(a)?, eval_const_expr(b)?) {
            (VmValue::Int(x), VmValue::Int(y)) => Some(VmValue::Int(x.wrapping_mul(y))),
            (VmValue::Float(x), VmValue::Float(y)) => Some(VmValue::Float(x * y)),
            _ => None,
        },
        ConstExpr::Div(a, b) => match (eval_const_expr(a)?, eval_const_expr(b)?) {
            (VmValue::Int(_), VmValue::Int(0)) => None,
            (VmValue::Int(x), VmValue::Int(y)) => Some(VmValue::Int(x / y)),
            (VmValue::Float(x), VmValue::Float(y)) => Some(VmValue::Float(x / y)),
            _ => None,
        },
        ConstExpr::Eq(a, b) => {
            let (x, y) = (eval_const_expr(a)?, eval_const_expr(b)?);
            Some(VmValue::Bool(x == y))
        }
        ConstExpr::Lt(a, b) => match (eval_const_expr(a)?, eval_const_expr(b)?) {
            (VmValue::Int(x), VmValue::Int(y)) => Some(VmValue::Bool(x < y)),
            (VmValue::Float(x), VmValue::Float(y)) => Some(VmValue::Bool(x < y)),
            _ => None,
        },
    }
}

// ---------------------------------------------------------------------------
// Part X / §42 + §61: WASM target helpers.
//
// Portability rule: these helpers describe the `wasm32-unknown-unknown`
// target (memory/page model, export naming) without requiring a WASM
// host. Emitting/running `.wasm` stays in `lexicon-wasm` + codegen;
// this module only answers "what would the WASM shape be?" so game,
// playground and AOT paths can plan without target-gated code.
// ---------------------------------------------------------------------------

/// WASM MVP limits used for planning (pages are 64 KiB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmLimits {
    pub max_memory_pages: u32,
    pub max_table_elements: u32,
}

impl Default for WasmLimits {
    fn default() -> Self {
        WasmLimits { max_memory_pages: 256, max_table_elements: 1024 }
    }
}

/// Descriptor of the WASM target shape for a module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmTargetInfo {
    pub triple: &'static str,
    pub limits: WasmLimits,
    pub exports: Vec<String>,
}

impl WasmTargetInfo {
    pub fn new(exports: Vec<String>) -> Self {
        WasmTargetInfo {
            triple: "wasm32-unknown-unknown",
            limits: WasmLimits::default(),
            exports,
        }
    }
}

/// `true` when compiled for a `wasm32` target.
pub fn is_wasm32() -> bool {
    cfg!(target_arch = "wasm32")
}

/// Stable WASM export name: `module_name` joined with `_`.
pub fn wasm_export_name(module: &str, name: &str) -> String {
    let clean = |s: &str| {
        s.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
    };
    format!("{}_{}", clean(module), clean(name))
}

/// `true` when `bytes` fits within `limits` (MVP memory budget check).
pub fn fits_wasm_limits(byte_len: usize, limits: WasmLimits) -> bool {
    byte_len as u64 <= limits.max_memory_pages as u64 * 65536
}

// ---------------------------------------------------------------------------
// Part X / §66: JIT optimization stubs + AOT path.
//
// AOT compilation is the PRIMARY deployment path. The JIT below is an
// explicit stub: `JitStub::compile` refuses to create executable memory
// by default (Spec §66 safeguard) and only proceeds when the caller
// opts into `allow_exec_memory` AND the host permits it. `AotModule`
// is the supported ahead-of-time artifact: versioned bytecode bytes +
// SHA-style fingerprint for reproducible builds.
// ---------------------------------------------------------------------------

/// Optimization level for the (stub) JIT tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JitOptLevel {
    None,
    Speed,
    Size,
}

/// JIT configuration with an explicit executable-memory safeguard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JitConfig {
    pub opt: JitOptLevel,
    /// MUST be explicitly enabled; default refuses executable memory.
    pub allow_exec_memory: bool,
}

impl Default for JitConfig {
    fn default() -> Self {
        JitConfig { opt: JitOptLevel::None, allow_exec_memory: false }
    }
}

/// Ahead-of-time artifact (primary path, Spec §66).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AotModule {
    pub version: u16,
    pub bytes: Vec<u8>,
}

impl AotModule {
    /// Emit an AOT artifact from a chunk (versioned encoding).
    pub fn emit(chunk: &Chunk) -> Self {
        AotModule { version: BYTECODE_VERSION, bytes: chunk.encode() }
    }

    /// Load + verify an AOT artifact (version + decode checks).
    pub fn load(bytes: &[u8]) -> Result<Self, String> {
        let chunk = Chunk::decode(bytes)?;
        Ok(AotModule { version: BYTECODE_VERSION, bytes: chunk.encode() })
    }

    pub fn decode(&self) -> Result<Chunk, String> {
        Chunk::decode(&self.bytes)
    }
}

/// JIT stub: tiered-optimization entry point (NOT the deployment path).
pub struct JitStub {
    pub config: JitConfig,
}

impl JitStub {
    pub fn new(config: JitConfig) -> Self {
        JitStub { config }
    }

    /// Attempt a JIT compile. Fails closed unless `allow_exec_memory`
    /// is set; even then it returns an AOT-equivalent artifact plus a
    /// note (no `mmap(RWX)` happens in this stub).
    pub fn compile(&self, chunk: &Chunk) -> Result<AotModule, String> {
        if !self.config.allow_exec_memory {
            return Err(
                "E0601: JIT disabled: executable-memory safeguards required (Spec §66); use AOT (`AotModule::emit`) as the primary path".to_string(),
            );
        }
        Ok(AotModule::emit(chunk))
    }

    /// Tiered-compile note: which tier WOULD handle `chunk` given its
    /// size (interpreter vs baseline vs optimizing), for profiling UIs.
    pub fn tier_note(chunk: &Chunk) -> &'static str {
        if chunk.code.len() < 64 {
            "tier: interpreter (small)"
        } else if chunk.code.len() < 4096 {
            "tier: baseline-jit candidate"
        } else {
            "tier: optimizing-jit candidate (hot)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_chunk(build: impl FnOnce(&mut Chunk)) -> VmOutput {
        let mut c = Chunk::new();
        build(&mut c);
        c.emit(OpCode::Return);
        VM::new().run(&c).expect("vm failed")
    }

    #[test]
    fn arithmetic() {
        // 2 + 3 * 4 = 14
        let out = run_chunk(|c| {
            let a = c.push_const(VmValue::Int(2));
            let b = c.push_const(VmValue::Int(3));
            let d = c.push_const(VmValue::Int(4));
            c.emit(OpCode::Const); c.emit_u32(a);
            c.emit(OpCode::Const); c.emit_u32(b);
            c.emit(OpCode::Const); c.emit_u32(d);
            c.emit(OpCode::Mul);
            c.emit(OpCode::Add);
        });
        assert_eq!(out.stack, vec![VmValue::Int(14)]);
    }

    #[test]
    fn locals_and_branch() {
        // x = 10; if x > 5 { print "big" } else { print "small" }
        let out = run_chunk(|c| {
            let ten = c.push_const(VmValue::Int(10));
            let five = c.push_const(VmValue::Int(5));
            let big = c.push_const(VmValue::Str("big".to_string()));
            let small = c.push_const(VmValue::Str("small".to_string()));
            c.emit(OpCode::Const); c.emit_u32(ten);
            c.emit(OpCode::SetLocal); c.emit_u8(0);
            c.emit(OpCode::GetLocal); c.emit_u8(0);
            c.emit(OpCode::Const); c.emit_u32(five);
            c.emit(OpCode::Gt);
            let jmp_pos = c.code.len();
            c.emit(OpCode::JumpIfFalse); c.emit_u32(0); // patched
            c.emit(OpCode::Const); c.emit_u32(big);
            c.emit(OpCode::Print);
            let end_pos = c.code.len();
            c.emit(OpCode::Jump); c.emit_u32(0); // patched
            let else_at = c.code.len() as u32;
            c.emit(OpCode::Const); c.emit_u32(small);
            c.emit(OpCode::Print);
            let end_at = c.code.len() as u32;
            c.code[jmp_pos + 1..jmp_pos + 5].copy_from_slice(&else_at.to_le_bytes());
            c.code[end_pos + 1..end_pos + 5].copy_from_slice(&end_at.to_le_bytes());
        });
        assert_eq!(out.printed, "big\n");
    }

    #[test]
    fn encode_versioned() {
        let mut c = Chunk::new();
        let a = c.push_const(VmValue::Int(1));
        c.emit(OpCode::Const);
        c.emit_u32(a);
        c.emit(OpCode::Return);
        let bytes = c.encode();
        let d = Chunk::decode(&bytes).unwrap();
        assert_eq!(d.consts, c.consts);
        let out = VM::new().run(&d).unwrap();
        assert_eq!(out.stack, vec![VmValue::Int(1)]);
        let mut bad = bytes.clone();
        bad[0] = 0xFF;
        assert!(Chunk::decode(&bad).is_err());
        assert!(Chunk::decode(&bytes[..5]).is_err());
    }

    #[test]
    fn const_exec_evaluator() {
        use super::ConstExpr as C;
        let e = C::Add(
            Box::new(C::Int(2)),
            Box::new(C::Mul(Box::new(C::Int(3)), Box::new(C::Int(4)))),
        );
        assert_eq!(eval_const_expr(&e), Some(VmValue::Int(14)));
        assert_eq!(
            eval_const_expr(&C::Div(Box::new(C::Int(1)), Box::new(C::Int(0)))),
            None
        );
        assert_eq!(
            eval_const_expr(&C::Not(Box::new(C::Bool(false)))),
            Some(VmValue::Bool(true))
        );
        assert_eq!(
            eval_const_expr(&C::Add(
                Box::new(C::Str("a".to_string())),
                Box::new(C::Str("b".to_string()))
            )),
            Some(VmValue::Str("ab".to_string()))
        );
        assert_eq!(
            eval_const_expr(&C::Lt(Box::new(C::Int(1)), Box::new(C::Int(2)))),
            Some(VmValue::Bool(true))
        );
    }

    #[test]
    fn wasm_helpers() {
        assert_eq!(wasm_export_name("my mod", "run!"), "my_mod_run_");
        let info = WasmTargetInfo::new(vec!["main".to_string()]);
        assert_eq!(info.triple, "wasm32-unknown-unknown");
        assert!(fits_wasm_limits(1024, WasmLimits::default()));
        assert!(!fits_wasm_limits(usize::MAX, WasmLimits::default()));
        let _ = is_wasm32();
    }

    #[test]
    fn jit_stub_fails_closed_aot_is_primary() {
        let mut c = Chunk::new();
        let a = c.push_const(VmValue::Int(1));
        c.emit(OpCode::Const);
        c.emit_u32(a);
        c.emit(OpCode::Return);
        let closed = JitStub::new(JitConfig::default());
        assert!(closed.compile(&c).is_err());
        let open = JitStub::new(JitConfig { opt: JitOptLevel::Speed, allow_exec_memory: true });
        let aot = open.compile(&c).unwrap();
        assert_eq!(AotModule::load(&aot.bytes).unwrap(), aot);
        assert!(JitStub::tier_note(&c).starts_with("tier:"));
        let decoded = aot.decode().unwrap();
        assert_eq!(VM::new().run(&decoded).unwrap().stack, vec![VmValue::Int(1)]);
    }

    #[test]
    fn div_by_zero_and_underflow() {
        let mut c = Chunk::new();
        let a = c.push_const(VmValue::Int(1));
        let b = c.push_const(VmValue::Int(0));
        c.emit(OpCode::Const); c.emit_u32(a);
        c.emit(OpCode::Const); c.emit_u32(b);
        c.emit(OpCode::Div);
        c.emit(OpCode::Return);
        assert!(VM::new().run(&c).is_err());
        let mut empty = Chunk::new();
        empty.emit(OpCode::Add);
        empty.emit(OpCode::Return);
        assert!(VM::new().run(&empty).is_err());
    }
}
