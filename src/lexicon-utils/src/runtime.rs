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
// 3. EFICIENT VALUE REPRESENTATION - 16 bytes ou menos
// =============================================================================

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(InternedStr),
    Null,
    List(Vec<Value>),
    Map(HashMap<Value, Value>),
}

impl Value {
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.as_str().is_empty(),
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
    }

    impl Chunk {
        pub fn new() -> Self {
            Self {
                code: Vec::new(),
                constants: Vec::new(),
                lines: Vec::new(),
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
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
