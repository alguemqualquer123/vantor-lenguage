// Optimized Runtime for Lexicon
// High-performance execution engine

use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::rc::Rc;

#[cfg(feature = "vm")]
use std::sync::Arc;

// =============================================================================
// 1. BUFFERED OUTPUT - Elimina flush por chamada
// =============================================================================

pub struct BufferedOutput {
    writer: BufWriter<io::Stdout>,
    line_count: usize,
    flush_threshold: usize,
}

impl BufferedOutput {
    pub fn new(capacity: usize) -> Self {
        Self {
            writer: BufWriter::with_capacity(capacity, io::stdout()),
            line_count: 0,
            flush_threshold: 1000, // Flush a cada 1000 linhas
        }
    }

    #[inline]
    pub fn write_line(&mut self, value: &str) {
        let _ = writeln!(self.writer, "{}", value);
        self.line_count += 1;

        if self.line_count >= self.flush_threshold {
            self.flush();
            self.line_count = 0;
        }
    }

    #[inline]
    pub fn write(&mut self, value: &str) {
        let _ = self.writer.write_all(value.as_bytes());
    }

    pub fn flush(&mut self) {
        let _ = self.writer.flush();
    }

    pub fn flush_if_needed(&mut self) {
        if self.line_count > 0 {
            self.flush();
            self.line_count = 0;
        }
    }
}

impl Default for BufferedOutput {
    fn default() -> Self {
        Self::new(65536) // 64KB buffer
    }
}

impl Drop for BufferedOutput {
    fn drop(&mut self) {
        self.flush_if_needed();
    }
}

// =============================================================================
// 2. STRING INTERNING - Evita alocação duplicada
// =============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InternedStr(Rc<str>);

impl InternedStr {
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const str {
        Rc::as_ptr(&self.0)
    }
}

impl From<String> for InternedStr {
    fn from(s: String) -> Self {
        InternedStr(Rc::from(s))
    }
}

impl From<&str> for InternedStr {
    fn from(s: &str) -> Self {
        InternedStr(Rc::from(s))
    }
}

pub struct StringInterner {
    map: HashMap<Rc<str>, usize>,
    strings: Vec<Rc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    #[inline]
    pub fn intern(&mut self, s: &str) -> InternedStr {
        if let Some(idx) = self.map.get(s) {
            return InternedStr(Rc::clone(&self.strings[*idx]));
        }

        let interned = Rc::from(s);
        let idx = self.strings.len();
        self.map.insert(Rc::clone(&interned), idx);
        self.strings.push(Rc::clone(&interned));
        InternedStr(interned)
    }

    #[inline]
    pub fn get(&self, s: &str) -> Option<InternedStr> {
        self.map
            .get(s)
            .map(|&idx| InternedStr(Rc::clone(&self.strings[idx])))
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    #[inline]
    pub fn ptr_eq(a: &InternedStr, b: &InternedStr) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
// =============================================================================
// 3. BINARY BLOB SYSTEM - Safe binary data handling
// =============================================================================


// =============================================================================
// 3. BINARY BLOB SYSTEM - Safe binary data handling
// =============================================================================

/// A binary large object. Uses a shared buffer to allow zero-copy slicing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Blob(Rc<Vec<u8>>);

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Blob(Rc::new(data))
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Blob(Rc::new(data.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a slice of the blob. Since Blob is a wrapper around Rc, 
    /// this is essentially zero-copy.
    pub fn slice(&self, start: usize, end: usize) -> Option<&[u8]> {
        if start > end || end > self.0.len() {
            return None;
        }
        Some(&self.0[start..end])
    }
}

// =============================================================================
// 4. EFICIENT VALUE REPRESENTATION - 16 bytes ou menos
// =============================================================================

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(InternedStr),
    Blob(Blob),
    Null,
    // Boxed to keep `Value` ≤ 24 bytes (Spec §45 memory contract):
    // heap payloads live behind a pointer instead of inline.
    List(Box<Vec<Value>>),
    Map(Box<HashMap<Value, Value>>),
}

impl Value {
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.as_str().is_empty(),
            Value::Blob(b) => !b.as_bytes().is_empty(),
            Value::Null => false,
            Value::List(v) => !v.is_empty(),
            Value::Map(m) => !m.is_empty(),
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => Some(self.is_truthy()),
        }
    }

    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "i64",
            Value::Float(_) => "f64",
            Value::Bool(_) => "bool",
            Value::Str(_) => "String",
            Value::Blob(_) => "Blob",
            Value::Null => "null",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => Self::str_eq(a, b),
            (Value::Null, Value::Null) => true,
            (Value::Int(i), Value::Float(f)) => *i as f64 == *f,
            (Value::Float(f), Value::Int(i)) => *f == *i as f64,
            _ => false,
        }
    }
}

impl Value {
    #[inline]
    fn str_eq(a: &InternedStr, b: &InternedStr) -> bool {
        // Fast path: pointer equality
        if Rc::ptr_eq(&a.0, &b.0) {
            return true;
        }
        // Slow path: content comparison
        a.as_str() == b.as_str()
    }
}

// =============================================================================
// 4. VM WITH BYTECODE - Para loops eficientes
// =============================================================================

#[cfg(feature = "vm")]
mod vm {
    use super::{BufferedOutput, InternedStr, StringInterner, Value};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OpCode {
        Nop,
        LoadConst,   // Load constant index
        LoadVar,     // Load local variable
        StoreVar,    // Store local variable
        LoadGlobal,  // Load global variable
        StoreGlobal, // Store global variable
        Add,
        Sub,
        Mul,
        Div,
        Mod,
        Neg,
        Increment,
        Decrement,
        Compare,
        Jump,
        JumpIfFalse,
        JumpIfTrue,
        Call,
        Return,
        Print,
        PrintLn,
        Pop,
        Dup,
        Swap,
        LoadNull,
        LoadTrue,
        LoadFalse,
        Eq,
        Ne,
        Lt,
        Le,
        Gt,
        Ge,
        And,
        Or,
        Not,
        Halt,
    }

    #[derive(Clone)]
    pub struct Chunk {
        pub code: Vec<u8>,
        pub constants: Vec<Value>,
        pub lines: Vec<usize>,
        pub signature: Option<Vec<u8>>,
    }

    impl Chunk {
        pub fn new() -> Self {
            Self {
                code: Vec::new(),
                constants: Vec::new(),
                lines: Vec::new(),
                signature: None,
            }
        }

        #[inline]
        pub fn add_op(&mut self, op: OpCode, line: usize) {
            self.code.push(op as u8);
            self.lines.push(line);
        }

        #[inline]
        pub fn add_constant(&mut self, value: Value, line: usize) -> usize {
            let idx = self.constants.len();
            self.constants.push(value);
            self.code.push(OpCode::LoadConst as u8);
            self.code.push(idx as u8);
            self.lines.push(line);
            idx
        }

        #[inline]
        pub fn add_jump(&mut self, op: OpCode, target: usize, line: usize) -> usize {
            self.code.push(op as u8);
            let offset = self.code.len();
            self.code.push(0);
            self.code.push(0);
            self.code.push(0);
            self.code.push(0);
            self.lines.push(line);
            offset
        }

        #[inline]
        pub fn patch_jump(&mut self, offset: usize, target: usize) {
            let bytes = target.to_le_bytes();
            self.code[offset] = bytes[0];
            self.code[offset + 1] = bytes[1];
            self.code[offset + 2] = bytes[2];
            self.code[offset + 3] = bytes[3];
        }
    }

    impl Default for Chunk {
        fn default() -> Self {
            Self::new()
        }
    }

    pub struct VM {
        pub stack: Vec<Value>,
        pub globals: HashMap<usize, Value>,
        pub constants: Vec<Value>,
        pub output: BufferedOutput,
        pub interner: StringInterner,
        ip: usize,
    }

    impl VM {
        pub fn new() -> Self {
            Self {
                stack: Vec::with_capacity(256),
                globals: HashMap::new(),
                constants: Vec::new(),
                output: BufferedOutput::new(65536),
                interner: StringInterner::new(),
                ip: 0,
            }
        }

        /// Verifies the bytecode for safety and integrity.
        /// Returns Ok(()) if the chunk is safe to execute, or an error message.
        pub fn verify_bytecode(&self, chunk: &Chunk) -> Result<(), String> {
            // 1. Integrity Check (Signature)
            if let Some(sig) = &chunk.signature {
                // In a real scenario, we'd verify against a trusted public key.
                // For now, we ensure the signature is not empty.
                if sig.is_empty() {
                    return Err("E0702: Bytecode signature is empty".to_string());
                }
            }

            // 2. Static Analysis Pass
            let mut stack_depth = 0;
            let mut ip = 0;

            while ip < chunk.code.len() {
                let op = chunk.code[ip];
                ip += 1;

                match op {
                    // Ops that push to stack
                    OpCode::LoadConst | OpCode::LoadVar | OpCode::LoadGlobal | 
                    OpCode::LoadNull | OpCode::LoadTrue | OpCode::LoadFalse => {
                        stack_depth += 1;
                        if op == OpCode::LoadConst || op == OpCode::LoadVar || 
                           op == OpCode::LoadGlobal {
                            ip += 1; // skip index
                        }
                    }
                    // Ops that pop 2 and push 1
                    OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod | 
                    OpCode::Eq | OpCode::Ne | OpCode::Lt | OpCode::Le | OpCode::Gt | 
                    OpCode::Ge | OpCode::And | OpCode::Or => {
                        if stack_depth < 2 {
                            return Err(format!("E0703: Stack underflow at IP {}", ip - 1));
                        }
                        // depth: -2 + 1 = -1
                    }
                    // Ops that pop 1 and push 1
                    OpCode::Neg | OpCode::Not => {
                        if stack_depth < 1 {
                            return Err(format!("E0703: Stack underflow at IP {}", ip - 1));
                        }
                    }
                    // Ops that pop 1
                    OpCode::StoreVar | OpCode::StoreGlobal | OpCode::Pop | OpCode::Return => {
                        if stack_depth < 1 {
                            return Err(format!("E0703: Stack underflow at IP {}", ip - 1));
                        }
                        stack_depth -= 1;
                        if op == OpCode::StoreVar { ip += 1; }
                        if op == OpCode::StoreGlobal { ip += 2; }
                    }
                    // Jumps
                    OpCode::Jump | OpCode::JumpIfFalse | OpCode::JumpIfTrue => {
                        if ip + 4 > chunk.code.len() {
                            return Err(format!("E0704: Jump operand out of bounds at IP {}", ip));
                        }
                        // We don't follow jumps in this simple linear pass to avoid infinite loops,
                        // but we verify the jump targets are within bounds.
                        let target = usize::from(chunk.code[ip]) << 24
                                    | usize::from(chunk.code[ip+1]) << 16
                                    | usize::from(chunk.code[ip+2]) << 8
                                    | usize::from(chunk.code[ip+3]);
                        if target > chunk.code.len() {
                            return Err(format!("E0704: Jump target {} out of bounds", target));
                        }
                        ip += 4;
                    }
                    // Other ops
                    OpCode::Nop | OpCode::Halt => {}
                    OpCode::Print | OpCode::PrintLn => {
                        if stack_depth < 1 {
                            return Err(format!("E0703: Stack underflow at IP {}", ip - 1));
                        }
                    }
                    OpCode::Dup => {
                        if stack_depth < 1 {
                            return Err(format!("E0703: Stack underflow at IP {}", ip - 1));
                        }
                        stack_depth += 1;
                    }
                    OpCode::Swap => {
                        if stack_depth < 2 {
                            return Err(format!("E0703: Stack underflow at IP {}", ip - 1));
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }

        #[inline]
        pub fn run(&mut self, chunk: &Chunk) {
            self.ip = 0;
            loop {
                if self.ip >= chunk.code.len() {
                    break;
                }

                let op = chunk.code[self.ip];
                self.ip += 1;

                match op {
                    OpCode::Nop => {}
                    OpCode::LoadConst => {
                        let idx = chunk.code[self.ip] as usize;
                        self.ip += 1;
                        self.stack.push(chunk.constants[idx].clone());
                    }
                    OpCode::LoadVar => {
                        let idx = chunk.code[self.ip] as usize;
                        self.ip += 1;
                        if let Some(v) = self.stack.get(self.stack.len().saturating_sub(idx + 1)) {
                            self.stack.push(v.clone());
                        }
                    }
                    OpCode::StoreVar => {
                        let idx = chunk.code[self.ip] as usize;
                        self.ip += 1;
                        let len = self.stack.len();
                        if idx + 1 <= len {
                            self.stack[len - idx - 1] = self.stack.last().unwrap().clone();
                        }
                    }
                    OpCode::LoadGlobal => {
                        let idx = usize::from(chunk.code[self.ip]) << 8
                            | usize::from(chunk.code[self.ip + 1]);
                        self.ip += 2;
                        self.stack
                            .push(self.globals.get(&idx).cloned().unwrap_or(Value::Null));
                    }
                    OpCode::StoreGlobal => {
                        let idx = usize::from(chunk.code[self.ip]) << 8
                            | usize::from(chunk.code[self.ip + 1]);
                        self.ip += 2;
                        let value = self.stack.last().unwrap().clone();
                        self.globals.insert(idx, value);
                    }
                    OpCode::Add => self.binary_op(|a, b| a + b),
                    OpCode::Sub => self.binary_op(|a, b| a - b),
                    OpCode::Mul => self.binary_op(|a, b| a * b),
                    OpCode::Div => self.binary_op(|a, b| a / b),
                    OpCode::Mod => self.binary_op(|a, b| a % b),
                    OpCode::Increment => {
                        if let Some(Value::Int(i)) = self.stack.last_mut() {
                            *i += 1;
                        }
                    }
                    OpCode::Decrement => {
                        if let Some(Value::Int(i)) = self.stack.last_mut() {
                            *i -= 1;
                        }
                    }
                    OpCode::Neg => {
                        if let Some(v) = self.stack.pop() {
                            self.stack.push(-v);
                        }
                    }
                    OpCode::Jump => {
                        let target = usize::from(chunk.code[self.ip]) << 24
                            | usize::from(chunk.code[self.ip + 1]) << 16
                            | usize::from(chunk.code[self.ip + 2]) << 8
                            | usize::from(chunk.code[self.ip + 3]);
                        self.ip = target;
                    }
                    OpCode::JumpIfFalse => {
                        let target = usize::from(chunk.code[self.ip]) << 24
                            | usize::from(chunk.code[self.ip + 1]) << 16
                            | usize::from(chunk.code[self.ip + 2]) << 8
                            | usize::from(chunk.code[self.ip + 3]);
                        self.ip += 4;
                        if !self.stack.last().map(|v| v.is_truthy()).unwrap_or(false) {
                            self.ip = target;
                        }
                    }
                    OpCode::JumpIfTrue => {
                        let target = usize::from(chunk.code[self.ip]) << 24
                            | usize::from(chunk.code[self.ip + 1]) << 16
                            | usize::from(chunk.code[self.ip + 2]) << 8
                            | usize::from(chunk.code[self.ip + 3]);
                        self.ip += 4;
                        if self.stack.last().map(|v| v.is_truthy()).unwrap_or(false) {
                            self.ip = target;
                        }
                    }
                    OpCode::Print => {
                        if let Some(v) = self.stack.last() {
                            self.output.write(&v.to_string());
                        }
                    }
                    OpCode::PrintLn => {
                        if let Some(v) = self.stack.last() {
                            self.output.write_line(&v.to_string());
                        }
                    }
                    OpCode::Pop => {
                        self.stack.pop();
                    }
                    OpCode::Dup => {
                        if let Some(v) = self.stack.last() {
                            self.stack.push(v.clone());
                        }
                    }
                    OpCode::LoadNull => {
                        self.stack.push(Value::Null);
                    }
                    OpCode::LoadTrue => {
                        self.stack.push(Value::Bool(true));
                    }
                    OpCode::LoadFalse => {
                        self.stack.push(Value::Bool(false));
                    }
                    OpCode::Eq => self.compare_op(|a, b| a == b),
                    OpCode::Ne => self.compare_op(|a, b| a != b),
                    OpCode::Lt => self.compare_op(|a, b| a < b),
                    OpCode::Le => self.compare_op(|a, b| a <= b),
                    OpCode::Gt => self.compare_op(|a, b| a > b),
                    OpCode::Ge => self.compare_op(|a, b| a >= b),
                    OpCode::And => self.binary_bool_op(|a, b| a && b),
                    OpCode::Or => self.binary_bool_op(|a, b| a || b),
                    OpCode::Not => {
                        if let Some(v) = self.stack.pop() {
                            self.stack.push(Value::Bool(!v.is_truthy()));
                        }
                    }
                    OpCode::Return => {
                        // Simplified: just clear stack to base
                        break;
                    }
                    OpCode::Halt => {
                        self.output.flush_if_needed();
                        break;
                    }
                    _ => {}
                }
            }
        }

        #[inline]
        fn binary_op<F>(&mut self, op: F)
        where
            F: FnOnce(Value, Value) -> Value,
        {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(op(a, b));
            }
        }

        #[inline]
        fn compare_op<F>(&mut self, op: F)
        where
            F: FnOnce(&Value, &Value) -> bool,
        {
            if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                self.stack.push(Value::Bool(op(&a, &b)));
            }
        }

        #[inline]
        fn binary_bool_op<F>(&mut self, op: F)
        where
            F: FnOnce(bool, bool) -> bool,
        {
            if let (Some(Value::Bool(b)), Some(Value::Bool(a))) =
                (self.stack.pop(), self.stack.pop())
            {
                self.stack.push(Value::Bool(op(a, b)));
            }
        }
    }

    impl Default for VM {
        fn default() -> Self {
            Self::new()
        }
    }

    // Implement arithmetic operations
    impl std::ops::Add for Value {
        type Output = Value;
        fn add(self, other: Value) -> Value {
            match (&self, &other) {
                (Value::Str(a), Value::Str(b)) => Value::Str(InternedStr::from(format!("{}{}", a.as_str(), b.as_str()))),
                (Value::Str(a), b) => Value::Str(InternedStr::from(format!("{}{}", a.as_str(), b.to_string()))),
                (a, Value::Str(b)) => Value::Str(InternedStr::from(format!("{}{}", a.to_string(), b.as_str()))),
                (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
                (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
                _ => Value::Null,
            }
        }
    }

    impl std::ops::Sub for Value {
        type Output = Value;
        fn sub(self, other: Value) -> Value {
            match (&self, &other) {
                (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - b),
                (Value::Float(a), Value::Int(b)) => Value::Float(a - *b as f64),
                _ => Value::Null,
            }
        }
    }

    impl std::ops::Mul for Value {
        type Output = Value;
        fn mul(self, other: Value) -> Value {
            match (&self, &other) {
                (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * b),
                (Value::Float(a), Value::Int(b)) => Value::Float(a * *b as f64),
                _ => Value::Null,
            }
        }
    }

    impl std::ops::Div for Value {
        type Output = Value;
        fn div(self, other: Value) -> Value {
            match (&self, &other) {
                (Value::Int(a), Value::Int(b)) if *b != 0 => Value::Int(a / b),
                (Value::Float(a), Value::Float(b)) if *b != 0.0 => Value::Float(a / b),
                (Value::Int(a), Value::Float(b)) if *b != 0.0 => Value::Float(*a as f64 / b),
                (Value::Float(a), Value::Int(b)) if *b != 0 => Value::Float(a / *b as f64),
                _ => Value::Null,
            }
        }
    }

    impl std::ops::Rem for Value {
        type Output = Value;
        fn rem(self, other: Value) -> Value {
            match (&self, &other) {
                (Value::Int(a), Value::Int(b)) if *b != 0 => Value::Int(a % b),
                (Value::Float(a), Value::Float(b)) if *b != 0.0 => Value::Float(a % b),
                _ => Value::Null,
            }
        }
    }

    impl std::ops::Neg for Value {
        type Output = Value;
        fn neg(self) -> Value {
            match self {
                Value::Int(i) => Value::Int(-i),
                Value::Float(f) => Value::Float(-f),
                _ => Value::Null,
            }
        }
    }

    impl ToString for Value {
        fn to_string(&self) -> String {
            match self {
                Value::Int(i) => i.to_string(),
                Value::Float(f) => {
                    if f.fract() == 0.0 {
                        format!("{:.1}", f)
                    } else {
                        f.to_string()
                    }
                }
                Value::Bool(b) => b.to_string(),
                Value::Str(s) => s.as_str().to_string(),
                Value::Null => "null".to_string(),
                Value::List(v) => {
                    let items: Vec<String> = v.iter().map(|x| x.to_string()).collect();
                    format!("[{}]", items.join(", "))
                }
                Value::Map(m) => {
                    let items: Vec<String> = m
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
                        .collect();
                    format!("{{{}}}", items.join(", "))
                }
            }
        }
    }
}

// =============================================================================
// 5. FAST LOOP EXECUTION - Para loops for i in 0..N
// =============================================================================

pub struct FastLoopExecutor {
    output: BufferedOutput,
    interner: StringInterner,
}

impl FastLoopExecutor {
    pub fn new() -> Self {
        Self {
            output: BufferedOutput::new(65536),
            interner: StringInterner::new(),
        }
    }

    /// Executa um loop for i in 0..N sem alocação por iteração
    #[inline]
    pub fn execute_for_range(&mut self, start: i64, end: i64, mut body: impl FnMut(i64)) {
        // Overflow protection
        if start >= end {
            return;
        }

        // Pre-allocate counter to avoid allocation in loop
        let mut i = start;
        while i < end {
            body(i);
            i += 1;
        }
    }

    /// Executa um loop com acumulador
    #[inline]
    pub fn execute_for_range_sum(&mut self, start: i64, end: i64) -> i64 {
        let mut sum: i64 = 0;
        let mut i = start;
        while i < end {
            sum += i;
            i += 1;
        }
        sum
    }

    /// Executa for com print (otimizado)
    #[inline]
    pub fn execute_for_range_with_print(&mut self, start: i64, end: i64, prefix: &str) {
        let mut i = start;
        while i < end {
            self.output.write_line(&format!("{}{}", prefix, i));
            i += 1;
        }
    }
}

impl Default for FastLoopExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FastLoopExecutor {
    fn drop(&mut self) {
        self.output.flush_if_needed();
    }
}

// =============================================================================
// BENCHMARK UTILITIES
// =============================================================================

pub mod benchmark {
    use std::time::Instant;

    pub struct Benchmark {
        start: Instant,
        name: String,
    }

    pub fn start(name: &str) -> Benchmark {
        Benchmark {
            start: Instant::now(),
            name: name.to_string(),
        }
    }

    impl Drop for Benchmark {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed();
            println!(
                "[{}] {} ns ({} μs)",
                self.name,
                elapsed.as_nanos(),
                elapsed.as_secs_f64() * 1_000_000.0
            );
        }
    }

    pub fn measure<F>(_name: &str, f: F) -> std::time::Duration
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        f();
        start.elapsed()
    }

    pub fn measure_iterations<F>(_name: &str, iterations: u32, mut f: F) -> std::time::Duration
    where
        F: FnMut(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        start.elapsed()
    }
}

// =============================================================================
// GC STATISTICS & TUNING (Spec §6)
// =============================================================================

/// Telemetry for the tracing collector. The runtime minimizes pause time;
/// these counters let production code diagnose memory pressure.
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    pub collections: u64,
    pub allocated_bytes: u64,
    pub reclaimed_bytes: u64,
    pub heap_bytes: u64,
    pub last_pause_micros: u64,
    pub max_pause_micros: u64,
}

impl GcStats {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_collection(&mut self, reclaimed: u64, pause_micros: u64) {
        self.collections += 1;
        self.reclaimed_bytes += reclaimed;
        self.last_pause_micros = pause_micros;
        if pause_micros > self.max_pause_micros {
            self.max_pause_micros = pause_micros;
        }
    }
    pub fn allocation_rate_per_sec(&self, elapsed_secs: f64) -> f64 {
        if elapsed_secs <= 0.0 { 0.0 } else { self.allocated_bytes as f64 / elapsed_secs }
    }
}

/// Heap growth policy hook (Spec §6): decides whether to grow the heap
/// under memory pressure instead of collecting.
#[derive(Debug, Clone)]
pub struct HeapPolicy {
    pub initial_bytes: usize,
    pub max_bytes: usize,
    pub growth_factor: f64,
}

impl Default for HeapPolicy {
    fn default() -> Self {
        HeapPolicy { initial_bytes: 8 << 20, max_bytes: 512 << 20, growth_factor: 2.0 }
    }
}

impl HeapPolicy {
    pub fn should_grow(&self, heap_bytes: usize, pressure: f64) -> bool {
        pressure > 0.8 && (heap_bytes as f64 * self.growth_factor) as usize <= self.max_bytes
    }
}

// =============================================================================
// STRINGS & UNICODE (Spec §9)
// =============================================================================

/// UTF-8 helpers: safe indexing, rune iteration, builders, hex/base64.
/// Grapheme clustering / normalization hooks are stubbed (SHOULD).
pub mod strings {
    /// Iterate Unicode scalar values with byte offsets.
    pub fn runes(s: &str) -> Vec<(usize, char)> {
        s.char_indices().collect()
    }
    /// Safe character indexing: returns None instead of panicking.
    pub fn char_at(s: &str, index: usize) -> Option<char> {
        s.chars().nth(index)
    }
    /// Safe slicing on character boundaries; None when out of range.
    pub fn slice_chars(s: &str, start: usize, end: usize) -> Option<String> {
        if start > end { return None; }
        let chars: Vec<char> = s.chars().collect();
        if end > chars.len() { return None; }
        Some(chars[start..end].iter().collect())
    }
    /// Efficient concatenation via reserved buffer.
    pub fn concat(parts: &[&str]) -> String {
        let len: usize = parts.iter().map(|p| p.len()).sum();
        let mut out = String::with_capacity(len);
        for p in parts { out.push_str(p); }
        out
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    pub fn hex_encode(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0xF) as usize] as char);
        }
        out
    }
    pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
        if s.len() % 2 != 0 { return None; }
        let digit = |c: u8| -> Option<u8> {
            match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            }
        };
        let bytes = s.as_bytes();
        let mut out = Vec::with_capacity(bytes.len() / 2);
        for pair in bytes.chunks(2) {
            out.push((digit(pair[0])? << 4) | digit(pair[1])?);
        }
        Some(out)
    }
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    pub fn base64_encode(bytes: &[u8]) -> String {
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let mut n: u32 = 0;
            for (i, b) in chunk.iter().enumerate() {
                n |= (*b as u32) << (16 - 8 * i);
            }
            let pad = 3 - chunk.len();
            for i in 0..4 - pad {
                out.push(B64[((n >> (18 - 6 * i)) & 63) as usize] as char);
            }
            for _ in 0..pad { out.push('='); }
        }
        out
    }
}

// =============================================================================
// OBSERVABILITY (Spec §29): structured logs, counters, tracing
// =============================================================================

/// Log level for structured logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel { Debug, Info, Warn, Error }

/// Minimal structured logger emitting JSON lines (Spec §29).
///
/// Ownership: owns `service` string; `log` borrows. Thread-safety:
/// `Send + Sync` (no interior mutability). Complexity: O(msg len).
pub struct StructuredLogger {
    pub level: LogLevel,
    pub service: String,
}

impl StructuredLogger {
    pub fn new(service: &str) -> Self {
        StructuredLogger { level: LogLevel::Info, service: service.to_string() }
    }

    /// Builder: set initial level. Complexity: O(1).
    pub fn with_level(service: &str, level: LogLevel) -> Self {
        StructuredLogger { level, service: service.to_string() }
    }

    /// Change the filter level at runtime. Complexity: O(1).
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn log(&self, level: LogLevel, correlation_id: &str, msg: &str) {
        let emit = match (self.level, level) {
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Debug) => false,
            _ => true,
        };
        if emit {
            println!(
                "{{\"service\":\"{}\",\"level\":\"{:?}\",\"correlation_id\":\"{}\",\"msg\":\"{}\"}}",
                self.service, level, correlation_id, msg
            );
        }
    }

    /// Convenience helpers. Complexity: O(msg len).
    pub fn debug(&self, correlation_id: &str, msg: &str) {
        self.log(LogLevel::Debug, correlation_id, msg);
    }
    pub fn info(&self, correlation_id: &str, msg: &str) {
        self.log(LogLevel::Info, correlation_id, msg);
    }
    pub fn warn(&self, correlation_id: &str, msg: &str) {
        self.log(LogLevel::Warn, correlation_id, msg);
    }
    pub fn error(&self, correlation_id: &str, msg: &str) {
        self.log(LogLevel::Error, correlation_id, msg);
    }
}

/// Counters / gauges / histograms (Spec §29).
///
/// Ownership: owns all series. Thread-safety: `!Sync` by convention —
/// keep one `Metrics` per thread/task or guard with a lock; sharing without
/// synchronization loses updates. Complexity: counter/gauge O(1);
/// histogram observe O(1) amortised.
#[derive(Debug, Default)]
pub struct Metrics {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    /// Raw histogram samples per name (bounded by caller; see
    /// `observe_histogram` cap). Each sample is one `f64`.
    pub histograms: HashMap<String, Vec<f64>>,
}

impl Metrics {
    pub fn new() -> Self { Self::default() }
    pub fn inc_counter(&mut self, name: &str, by: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += by;
    }
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    /// Max samples retained per histogram (DoS bound for untrusted labels).
    /// Ownership: n/a. Complexity: O(1).
    pub const MAX_HISTOGRAM_SAMPLES: usize = 10_000;

    /// Record one histogram sample, dropping the oldest when capped.
    /// Complexity: O(1) amortised.
    pub fn observe_histogram(&mut self, name: &str, value: f64) {
        let v = self.histograms.entry(name.to_string()).or_default();
        if v.len() >= Self::MAX_HISTOGRAM_SAMPLES {
            v.remove(0);
        }
        v.push(value);
    }

    /// `(count, sum, min, max)` for a histogram, or `None` when empty.
    /// Complexity: O(n).
    pub fn histogram_stats(&self, name: &str) -> Option<(usize, f64, f64, f64)> {
        let v = self.histograms.get(name)?;
        if v.is_empty() {
            return None;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;
        for x in v {
            if *x < min {
                min = *x;
            }
            if *x > max {
                max = *x;
            }
            sum += *x;
        }
        Some((v.len(), sum, min, max))
    }

    pub fn counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }

    pub fn gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).copied()
    }
}

// ---------------------------------------------------------------------------
// Health checks (Spec §29, observability).
//
// Ownership: registry owns boxed probes. Thread-safety: probes must be
// `Send + Sync`; `run_all` borrows. Complexity: O(checks).
// ---------------------------------------------------------------------------

/// One health probe result.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub name: String,
    pub ok: bool,
    pub latency_ms: u128,
    pub message: String,
}

#[allow(dead_code)]
impl HealthStatus {
    pub fn ok(name: &str, latency_ms: u128) -> Self {
        HealthStatus { name: name.to_string(), ok: true, latency_ms, message: "ok".to_string() }
    }

    pub fn fail(name: &str, latency_ms: u128, message: &str) -> Self {
        HealthStatus { name: name.to_string(), ok: false, latency_ms, message: message.to_string() }
    }
}

/// Registry of health probes (liveness/readiness).
#[allow(dead_code)]
#[derive(Default)]
pub struct HealthRegistry {
    checks: Vec<(String, Box<dyn Fn() -> Result<String, String> + Send + Sync>)>,
}

#[allow(dead_code)]
impl HealthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a probe. Complexity: O(1).
    pub fn register<F>(&mut self, name: &str, probe: F)
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        self.checks.push((name.to_string(), Box::new(probe)));
    }

    /// Run all probes, measuring latency each. Complexity: O(checks).
    pub fn run_all(&self) -> Vec<HealthStatus> {
        let mut out = Vec::with_capacity(self.checks.len());
        for (name, probe) in &self.checks {
            let t = std::time::Instant::now();
            match probe() {
                Ok(msg) => out.push(HealthStatus {
                    name: name.clone(),
                    ok: true,
                    latency_ms: t.elapsed().as_millis(),
                    message: msg,
                }),
                Err(e) => out.push(HealthStatus {
                    name: name.clone(),
                    ok: false,
                    latency_ms: t.elapsed().as_millis(),
                    message: e,
                }),
            }
        }
        out
    }

    /// True when every probe passes. Complexity: O(checks).
    pub fn is_healthy(&self) -> bool {
        self.run_all().iter().all(|s| s.ok)
    }
}

// =============================================================================
// I/O ABSTRACTIONS (Spec §18): Reader/Writer, buffered wrappers, streams
// =============================================================================
//
// Ownership: streams own their buffers/handles; adapters borrow the inner
// stream for exactly the call duration. Thread-safety: types are `Send`
// when the inner handle is `Send`; no internal locking — share behind a
// `Mutex` if needed. Complexity: documented per method.

use std::fs::File;
use std::io::Read as _;

/// Byte-source contract (Spec §18).
///
/// Ownership: implementors own or borrow the source; `read` borrows `&mut`.
/// Thread-safety: `Send` iff the source is `Send`.
/// Complexity: `read` O(buf.len()) max; may return short reads.
pub trait Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    /// Fill exactly `buf` or return `UnexpectedEof`. Complexity: O(buf.len()).
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let mut off = 0;
        while off < buf.len() {
            match self.read(&mut buf[off..]) {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "E0601: eof")),
                Ok(n) => off += n,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    /// Drain to `Vec`, capped at `max_bytes` (untrusted-input defense).
    /// Complexity: O(bytes).
    fn read_to_end_capped(&mut self, max_bytes: usize) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let n = self.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            if out.len() + n > max_bytes {
                return Err(io::Error::new(io::ErrorKind::OutOfMemory, "E0601: read cap exceeded"));
            }
            out.extend_from_slice(&chunk[..n]);
        }
        Ok(out)
    }
}

/// Byte-sink contract (Spec §18).
///
/// Ownership: implementors own or borrow the sink. Thread-safety: `Send`
/// iff the sink is `Send`. Complexity: `write` O(buf.len()) max.
pub trait Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn flush(&mut self) -> io::Result<()>;
    /// Write the whole buffer or fail. Complexity: O(buf.len()).
    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::WriteZero, "E0601: write zero")),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Reader for &[u8] {
    /// Complexity: O(buf.len()).
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = (*self).len().min(buf.len());
        buf[..n].copy_from_slice(&self[..n]);
        *self = &self[n..];
        Ok(n)
    }
}

impl Writer for Vec<u8> {
    /// Complexity: O(buf.len()) amortised.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Buffered reader wrapper (Spec §18).
///
/// Ownership: owns the inner reader + buffer. Thread-safety: `Send` iff `R`.
/// Complexity: amortised O(1) per byte; `fill` O(capacity).
pub struct BufferedReader<R> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
}

impl<R> BufferedReader<R> {
    pub fn new(inner: R, capacity: usize) -> Self {
        let capacity = capacity.max(1);
        BufferedReader { inner, buf: vec![0u8; capacity], pos: 0, cap: 0 }
    }

    pub fn inner(&self) -> &R {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Reader> BufferedReader<R> {
    /// Read one byte without extra syscalls when buffered. Complexity: O(1).
    pub fn read_byte(&mut self) -> io::Result<Option<u8>> {
        if self.pos >= self.cap {
            // refill
            let mut tmp = vec![0u8; self.buf.len()];
            // Use a temporary borrow to satisfy the borrow checker.
            let n = self.inner.read(&mut tmp)?;
            if n == 0 {
                return Ok(None);
            }
            self.buf[..n].copy_from_slice(&tmp[..n]);
            self.pos = 0;
            self.cap = n;
        }
        let b = self.buf[self.pos];
        self.pos += 1;
        Ok(Some(b))
    }
}

impl<R: Reader> Reader for BufferedReader<R> {
    /// Complexity: O(buf.len()).
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos < self.cap {
            let n = (self.cap - self.pos).min(buf.len());
            buf[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
            self.pos += n;
            return Ok(n);
        }
        self.inner.read(buf)
    }
}

/// Buffered writer wrapper (Spec §18).
///
/// Ownership: owns inner writer + buffer; `flush`/`drop` push buffered
/// bytes. Thread-safety: `Send` iff `W`. Complexity: amortised O(1)/byte.
pub struct BufferedWriter<W> {
    inner: W,
    buf: Vec<u8>,
    capacity: usize,
}

impl<W> BufferedWriter<W> {
    pub fn new(inner: W, capacity: usize) -> Self {
        BufferedWriter { inner, buf: Vec::new(), capacity: capacity.max(1) }
    }

    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }

    pub fn inner(&self) -> &W {
        &self.inner
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Writer> BufferedWriter<W> {
    /// Complexity: O(1) amortised; flush O(buffered).
    pub fn write_byte(&mut self, b: u8) -> io::Result<()> {
        self.buf.push(b);
        if self.buf.len() >= self.capacity {
            self.flush()?;
        }
        Ok(())
    }
}

impl<W: Writer> Writer for BufferedWriter<W> {
    /// Complexity: O(buf.len()) amortised.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buf.len() + buf.len() < self.capacity {
            self.buf.extend_from_slice(buf);
            return Ok(buf.len());
        }
        self.flush()?;
        self.inner.write(buf)
    }

    /// Complexity: O(buffered).
    /// Ownership: explicit flush required; `Drop` does NOT auto-flush
    /// because `Drop` cannot carry the `W: Writer` bound (E0367).
    fn flush(&mut self) -> io::Result<()> {
        if !self.buf.is_empty() {
            let pending = std::mem::take(&mut self.buf);
            self.inner.write_all(&pending)?;
        }
        self.inner.flush()
    }
}

/// In-memory seekable stream (Spec §18).
///
/// Ownership: owns `Vec<u8>` + cursor. Thread-safety: `Send + Sync`.
/// Complexity: read/write O(bytes); seek O(1).
#[derive(Debug, Clone, Default)]
pub struct MemoryStream {
    buf: Vec<u8>,
    pos: usize,
}

impl MemoryStream {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec(buf: Vec<u8>) -> Self {
        MemoryStream { buf, pos: 0 }
    }

    /// Complexity: O(1).
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Complexity: O(1).
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Clamp-seek; positions past the end pin to `len`. Complexity: O(1).
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos.min(self.buf.len());
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }
}

impl Reader for MemoryStream {
    /// Complexity: O(buf.len()).
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = (self.buf.len() - self.pos).min(buf.len());
        buf[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

impl Writer for MemoryStream {
    /// Complexity: O(buf.len()) amortised.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.pos == self.buf.len() {
            self.buf.extend_from_slice(buf);
        } else {
            let end = self.pos + buf.len();
            if end > self.buf.len() {
                self.buf.resize(end, 0);
            }
            self.buf[self.pos..end].copy_from_slice(buf);
        }
        self.pos += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// File-backed stream wrapper (Spec §18, §49).
///
/// Ownership: owns the `File` handle + path. Thread-safety: `Send` (handle
/// is `Send`); concurrent use needs external locking. Complexity: O(bytes).
pub struct FileStream {
    file: File,
    path: String,
}

impl FileStream {
    /// Open for reading. Complexity: O(1) syscall.
    pub fn open_read(path: &str) -> io::Result<Self> {
        Ok(FileStream { file: File::open(path)?, path: path.to_string() })
    }

    /// Create/truncate for writing. Complexity: O(1) syscall.
    pub fn open_write(path: &str) -> io::Result<Self> {
        Ok(FileStream { file: File::create(path)?, path: path.to_string() })
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Reader for FileStream {
    /// Complexity: O(buf.len()) syscall.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Writer for FileStream {
    /// Complexity: O(buf.len()) syscall.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

// =============================================================================
// OS / PROCESS SERVICES affiliados ao runtime (Spec §18 + §50)
//
// `crate::fsx` owns the full OS surface; these re-exports + thin helpers
// keep `runtime` usable as the single stdlib entry point without moving
// code (no API removal). Referenced `fsx` docs carry the canonical
// complexity/ownership contracts.
// =============================================================================

pub use crate::fsx::{
    basename, copy_file, create_dir_all, dirname, env_get, env_list, env_remove, env_set,
    extension, file_meta, get_perms, is_absolute, is_tty, join_path, list_dir, normalize_path,
    path_exists, read_file, remove_dir_all, remove_file, rename_path, set_readonly, sysinfo,
    temp_dir, tty_width_or, write_file, FileMeta, FilePerms, LexChild, Os, SysInfo,
};

/// Spawn a program and wait for its output (stdout captured).
/// Ownership: no handles retained. Thread-safety: `Send + Sync`.
/// Complexity: O(child runtime + output bytes).
pub fn run_capture(program: &str, args: &[&str]) -> Result<(i32, String), String> {
    let out = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("E0501: spawn {}: {}", program, e))?;
    let code = out.status.code().unwrap_or(-1);
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    if !out.stderr.is_empty() {
        text.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    Ok((code, text))
}

/// Current process id. Complexity: O(1).
pub fn pid() -> u32 {
    std::process::id()
}

// =============================================================================
// GAME CAPABILITY STUBS (Spec §61)
//
// No mandatory runtime: these are opt-in, heap-owned handles with explicit
// budgets. Real GPU/audio/input backends (Vulkan/DX/GL/Metal, WASAPI/ALSA,
// winit) are integration points; shapes below pin ownership, thread rules
// and error codes. All stubs are `#[allow(dead_code)]` by design.
// =============================================================================

/// Job-system stub: bounded task queue drained explicitly (no hidden threads).
///
/// Ownership: owns queued closures. Thread-safety: `!Sync`; drive from one
/// thread or guard with a lock. Complexity: dispatch O(1), `run_all` O(jobs).
#[allow(dead_code)]
pub mod game {
    /// Bounded job queue; `run_all` executes FIFO on the calling thread.
    pub struct JobSystem {
        queue: Vec<Box<dyn FnOnce() + Send>>,
        max_jobs: usize,
    }

    impl JobSystem {
        /// Complexity: O(1).
        pub fn new(max_jobs: usize) -> Self {
            JobSystem { queue: Vec::new(), max_jobs: max_jobs.max(1) }
        }

        /// Enqueue or reject when full. Complexity: O(1).
        pub fn dispatch<F>(&mut self, job: F) -> Result<(), String>
        where
            F: FnOnce() + Send + 'static,
        {
            if self.queue.len() >= self.max_jobs {
                return Err("E0601: job queue full".to_string());
            }
            self.queue.push(Box::new(job));
            Ok(())
        }

        /// Run all queued jobs on this thread. Complexity: O(jobs).
        pub fn run_all(&mut self) {
            for job in self.queue.drain(..) {
                job();
            }
        }

        pub fn pending(&self) -> usize {
            self.queue.len()
        }
    }

    /// Entity id (generational index stub: plain counter).
    pub type Entity = u64;

    /// Minimal ECS world stub: entities + typed `f32` components.
    ///
    /// Ownership: world owns all components. Thread-safety: `!Sync`.
    /// Complexity: spawn O(1); insert/get O(1).
    #[derive(Default)]
    pub struct EcsWorld {
        next: Entity,
        positions: std::collections::HashMap<Entity, [f32; 3]>,
    }

    impl EcsWorld {
        pub fn new() -> Self {
            Self::default()
        }

        /// Complexity: O(1).
        pub fn spawn(&mut self) -> Entity {
            self.next += 1;
            self.next
        }

        /// Complexity: O(1).
        pub fn set_position(&mut self, e: Entity, xyz: [f32; 3]) {
            self.positions.insert(e, xyz);
        }

        /// Complexity: O(1).
        pub fn position(&self, e: Entity) -> Option<[f32; 3]> {
            self.positions.get(&e).copied()
        }

        /// Complexity: O(1).
        pub fn despawn(&mut self, e: Entity) -> bool {
            self.positions.remove(&e).is_some()
        }

        pub fn entity_count(&self) -> u64 {
            self.next
        }
    }

    /// GPU buffer stub (vertex/uniform/...).
    ///
    /// Ownership: owns a heap copy; real backend would own device memory.
    /// Thread-safety: `Send` but NOT `Sync` (no internal locking).
    /// Complexity: upload O(bytes).
    pub struct GpuBuffer {
        label: String,
        bytes: Vec<u8>,
        capacity: usize,
    }

    impl GpuBuffer {
        /// Complexity: O(1).
        pub fn new(label: &str, capacity: usize) -> Result<Self, String> {
            if capacity > (256 << 20) {
                return Err("E0601: gpu buffer exceeds 256 MiB cap".to_string());
            }
            Ok(GpuBuffer { label: label.to_string(), bytes: Vec::new(), capacity })
        }

        /// Replace contents. Complexity: O(bytes).
        pub fn upload(&mut self, data: &[u8]) -> Result<(), String> {
            if data.len() > self.capacity {
                return Err("E0601: gpu upload exceeds capacity".to_string());
            }
            self.bytes.clear();
            self.bytes.extend_from_slice(data);
            Ok(())
        }

        pub fn len(&self) -> usize {
            self.bytes.len()
        }

        pub fn is_empty(&self) -> bool {
            self.bytes.is_empty()
        }

        pub fn label(&self) -> &str {
            &self.label
        }
    }

    /// Audio clip stub (PCM16 mono placeholder).
    ///
    /// Ownership: owns samples. Thread-safety: `Send + Sync`.
    /// Complexity: O(samples).
    #[derive(Debug, Clone)]
    pub struct AudioClip {
        pub sample_rate: u32,
        pub samples: Vec<i16>,
    }

    impl AudioClip {
        /// Silence of `len` samples. Complexity: O(len).
        pub fn silence(sample_rate: u32, len: usize) -> Self {
            AudioClip { sample_rate, samples: vec![0; len] }
        }

        pub fn duration_secs(&self) -> f64 {
            if self.sample_rate == 0 {
                0.0
            } else {
                self.samples.len() as f64 / self.sample_rate as f64
            }
        }
    }

    /// Input state stub (buttons + axis).
    ///
    /// Ownership: value type. Thread-safety: `Send + Sync`.
    /// Complexity: O(1).
    #[derive(Debug, Clone, Default)]
    pub struct InputState {
        pub buttons: u64,
        pub axis_x: f32,
        pub axis_y: f32,
    }

    impl InputState {
        /// Complexity: O(1).
        pub fn button(&self, index: u32) -> bool {
            if index >= 64 {
                return false;
            }
            (self.buttons >> index) & 1 == 1
        }

        /// Complexity: O(1).
        pub fn set_button(&mut self, index: u32, pressed: bool) {
            if index >= 64 {
                return;
            }
            if pressed {
                self.buttons |= 1 << index;
            } else {
                self.buttons &= !(1 << index);
            }
        }
    }

    /// Asset handle stub (hot-reload generation counter).
    ///
    /// Ownership: value type. Thread-safety: `Send + Sync`.
    /// Complexity: O(1).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AssetHandle {
        pub id: u64,
        pub generation: u64,
    }

    impl AssetHandle {
        pub fn is_stale(&self, current_generation: u64) -> bool {
            self.generation != current_generation
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intern;

    #[test]
    fn test_string_interning() {
        let mut interner = StringInterner::new();

        let a = interner.intern("hello");
        let b = interner.intern("hello");
        let c = intern!("world");

        assert!(StringInterner::ptr_eq(&a, &b));
        assert!(!StringInterner::ptr_eq(&a, &c));
    }

    #[test]
    fn test_value_size() {
        let size = std::mem::size_of::<Value>();
        assert!(size <= 24, "Value too large: {} bytes", size);
    }

    #[test]
    fn test_fast_loop() {
        let mut executor = FastLoopExecutor::new();

        let mut sum: i64 = 0;
        executor.execute_for_range(0, 1_000_000, |i| {
            sum += i;
        });

        let expected = (0i64..1_000_000).sum::<i64>();
        assert_eq!(sum, expected);
    }

    #[test]
    fn test_fast_loop_sum() {
        let mut executor = FastLoopExecutor::new();
        let sum = executor.execute_for_range_sum(0, 1_000_000);
        let expected = (0i64..1_000_000).sum::<i64>();
        assert_eq!(sum, expected);
    }

    #[test]
    fn test_buffered_output() {
        let mut output = BufferedOutput::new(4096);

        for i in 0..100 {
            output.write_line(&format!("Line {}", i));
        }

        // Não deve fazer flush automaticamente
        output.flush();
    }

    #[test]
    fn test_lexio_memory_and_buffered() {
        // MemoryStream roundtrip + seek.
        let mut m = MemoryStream::from_vec(b"hello".to_vec());
        let mut buf = [0u8; 5];
        assert_eq!(m.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"hello");
        assert_eq!(m.read(&mut buf).unwrap(), 0);
        m.seek(0);
        let mut out = BufferedWriter::new(MemoryStream::new(), 4);
        out.write_all(b"abcdef").unwrap();
        out.flush().unwrap();
        assert_eq!(out.inner().as_bytes(), b"abcdef");

        // BufferedReader over slice.
        let data: &[u8] = b"xy";
        let mut br = BufferedReader::new(data, 8);
        assert_eq!(br.read_byte().unwrap(), Some(b'x'));
        let mut rest = [0u8; 4];
        assert_eq!(br.read(&mut rest).unwrap(), 1);
        assert_eq!(rest[0], b'y');

        // Capped read rejects over-limit.
        let big: &[u8] = b"12345678";
        let mut r = big;
        assert!(r.read_to_end_capped(4).is_err());
    }

    #[test]
    fn test_metrics_histogram_and_health() {
        let mut m = Metrics::new();
        m.inc_counter("req", 2);
        m.set_gauge("mem", 1.5);
        m.observe_histogram("lat", 5.0);
        m.observe_histogram("lat", 15.0);
        assert_eq!(m.counter("req"), 2);
        assert_eq!(m.gauge("mem"), Some(1.5));
        let (n, sum, min, max) = m.histogram_stats("lat").unwrap();
        assert_eq!((n, min, max), (2, 5.0, 15.0));
        assert_eq!(sum, 20.0);
        assert!(m.histogram_stats("missing").is_none());

        let mut reg = HealthRegistry::new();
        reg.register("db", || Ok("connected".to_string()));
        reg.register("disk", || Err("full".to_string()));
        let all = reg.run_all();
        assert_eq!(all.len(), 2);
        assert!(!reg.is_healthy());
        let ok = HealthStatus::ok("x", 1);
        assert!(ok.ok);
    }

    #[test]
    fn test_os_helpers_and_game_stubs() {
        assert_eq!(join_path(&["a", "b"]), "a/b");
        assert_eq!(basename("a/b.txt"), "b.txt");
        assert_eq!(pid(), std::process::id());
        // run_capture on the test binary itself.
        let exe = std::env::current_exe().unwrap();
        let (code, _) = run_capture(&exe.to_string_lossy(), &["--list"]).unwrap();
        assert_eq!(code, 0);

        let mut js = game::JobSystem::new(2);
        js.dispatch(|| {}).unwrap();
        js.dispatch(|| {}).unwrap();
        assert!(js.dispatch(|| {}).is_err());
        assert_eq!(js.pending(), 2);
        js.run_all();
        assert_eq!(js.pending(), 0);

        let mut w = game::EcsWorld::new();
        let e = w.spawn();
        w.set_position(e, [1.0, 2.0, 3.0]);
        assert_eq!(w.position(e), Some([1.0, 2.0, 3.0]));
        assert!(w.despawn(e));

        let mut buf = game::GpuBuffer::new("v", 16).unwrap();
        buf.upload(b"data").unwrap();
        assert_eq!(buf.len(), 4);
        assert!(buf.upload(&[0u8; 17]).is_err());

        let clip = game::AudioClip::silence(44100, 44100);
        assert_eq!(clip.duration_secs(), 1.0);
        let mut input = game::InputState::default();
        input.set_button(1, true);
        assert!(input.button(1));
        assert!(!input.button(2));
        let h = game::AssetHandle { id: 1, generation: 2 };
        assert!(h.is_stale(3));
        assert!(!h.is_stale(2));
    }
}

// Helper macro para string interning
#[macro_export]
macro_rules! intern {
    ($s:expr) => {{
        use $crate::runtime::StringInterner;
        // Em tempo de compilação, strings literais são internadas automaticamente
        $crate::runtime::InternedStr::from($s)
    }};
}
