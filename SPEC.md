# VantorLang Specification

**Version:** 1.0.0  
**Status:** Design Draft  
**Implementation Target:** Rust

---

## Table of Contents

1. [Philosophy and Design Principles](#1-philosophy-and-design-principles)
2. [Syntax Specification](#2-syntax-specification)
3. [Type System](#3-type-system)
4. [Concurrency Model](#4-concurrency-model)
5. [Memory Management](#5-memory-management)
6. [Compiler Architecture](#6-compiler-architecture)
7. [Formal Grammar (EBNF)](#7-formal-grammar-ebnf)
8. [Build System and CLI](#8-build-system-and-cli)
9. [Package Management](#9-package-management)
10. [Testing Framework](#10-testing-framework)
11. [Code Examples](#11-code-examples)
12. [Technical Comparison](#12-technical-comparison)
13. [Compiler Implementation (Rust)](#13-compiler-implementation-rust)
14. [Development Roadmap](#14-development-roadmap)

---

## 1. Philosophy and Design Principles

### 1.1 Core Philosophy

VantorLang is designed as a **systems-level programming language** that combines the safety of modern type systems with the performance required for backend, systems, and desktop applications. Drawing inspiration from Java and C# syntax while adopting modern architectural decisions from Rust and modern functional languages.

### 1.2 Design Principles

| Principle | Description |
|-----------|-------------|
| **Type Safety** | All types are statically checked at compile-time with optional runtime assertions |
| **Null Safety** | Nullable types are explicit (`T?`) with mandatory null-checks |
| **Memory Safety** | Ownership model with borrow checking, preventing use-after-free and data races |
| **Performance** | Zero-cost abstractions, inline optimization, native code generation |
| **Concurrency** | Modern async/await with structured concurrency and actor-based message passing |
| **Interoperability** | Foreign Function Interface (FFI) for C and Rust libraries |
| **Developer Experience** | Clear error messages, IDE support, fast compilation |

### 1.3 Design Goals

- **Primary:** Backend services, CLI tools, systems programming
- **Secondary:** Desktop applications, embedded systems (with constraints)
- **Non-goals:** Web frontend (use JS/TS), extremely constrained embedded (use C)

---

## 2. Syntax Specification

### 2.1 File Structure

```
project/
├── src/
│   ├── main.vnt
│   ├── module/
│   │   └── handler.vnt
│   └── package.vnt
├── tests/
├── resources/
├── vantor.toml
└── build/
```

### 2.2 Module Declaration

```vantor
module core.system;

import core.io.Console;
import core.io.File;
import core.net.Http;
import core.collections.List;

pub fn main() -> void {
    Console.writeLine("Hello, VantorLang!");
}
```

### 2.3 Visibility Modifiers

| Modifier | Description |
|----------|-------------|
| `pub` | Public - accessible everywhere |
| `pub(crate)` | Module-public - accessible within crate |
| `private` | Private - accessible within struct/class |
| `protected` | Protected - accessible in subclasses |

### 2.4 Keywords

```
module  import  pub     private  protected
class   struct  enum    trait    interface
fn      let     var     mut      const
if      else    match   for      while  loop
return  break   continue  defer   yield
async   await   task    channel  actor
try     catch   throw   finally
true    false   null    self     super
type    where   as      is       in
unsafe  extern  inline  native
```

---

## 3. Type System

### 3.1 Primitive Types

| Type | Size | Description |
|------|------|-------------|
| `bool` | 1 byte | `true` or `false` |
| `char` | 4 bytes | UTF-32 Unicode scalar value |
| `i8` | 1 byte | Signed 8-bit integer |
| `i16` | 2 bytes | Signed 16-bit integer |
| `i32` | 4 bytes | Signed 32-bit integer (default for `int`) |
| `i64` | 8 bytes | Signed 64-bit integer |
| `i128` | 16 bytes | Signed 128-bit integer |
| `u8` | 1 byte | Unsigned 8-bit integer |
| `u16` | 2 bytes | Unsigned 16-bit integer |
| `u32` | 4 bytes | Unsigned 32-bit integer |
| `u64` | 8 bytes | Unsigned 64-bit integer |
| `u128` | 16 bytes | Unsigned 128-bit integer |
| `f32` | 4 bytes | IEEE 754 32-bit floating point |
| `f64` | 8 bytes | IEEE 754 64-bit floating point (default for `float`) |
| `decimal` | 16 bytes | High-precision decimal for financial calculations |
| `void` | 0 bytes | No return value |

### 3.2 Type Aliases

```vantor
type int = i32;
type long = i64;
type float = f64;
type double = f64;
type String = collections.String;
type List<T> = collections.List<T>;
type Map<K, V> = collections.Map<K, V>;
```

### 3.3 Nullable Types

All reference types are non-nullable by default. Use `T?` for nullable:

```vantor
let name: String = "Alice";      // Non-nullable
let middleName: String? = null; // Nullable

// Safe access with null-coalescing
let length = middleName?.length ?? 0;

// Pattern matching
match middleName {
    Some(n) => Console.writeLine(n),
    None => Console.writeLine("No middle name"),
}
```

### 3.4 Option<T> and Result<T, E>

```vantor
// Option<T> - represents optional value
let user = findUser(123);
match user {
    Some(u) => Console.writeLine(u.name),
    None => Console.writeLine("User not found"),
}

// Result<T, E> - represents success or error
let result = parseNumber("42");
match result {
    Ok(n) => Console.writeLine(n),
    Err(e) => Console.writeLine("Parse error: " + e.message),
}
```

### 3.5 Collections

```vantor
// Arrays - fixed size, stack-allocated for primitives
let numbers: [i32; 3] = [1, 2, 3];

// List<T> - dynamic array
let names = List<String>::new();
names.add("Alice");
names.add("Bob");

// Map<K, V> - key-value hash map
let scores = Map<String, i32>::new();
scores.insert("Alice", 100);
scores.insert("Bob", 95);

// Set<T>
let unique = Set<i32>::new();
```

### 3.6 Generics

```vantor
class Container<T> {
    private value: T;
    
    pub fn new(value: T) -> Container<T> {
        Container { value }
    }
    
    pub fn get(&self) -> T {
        self.value
    }
    
    pub fn map<U>(self, f: fn(T) -> U) -> Container<U> {
        Container::new(f(self.value))
    }
}

// Generic constraints
fn max<T: Comparable<T>>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn process<T: Clone + Debug>(items: List<T>) -> void {
    // ...
}
```

### 3.7 Structs (Value Types)

```vantor
struct Point {
    x: f64,
    y: f64,
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8 = 255,  // Default value
}

let p = Point { x: 10.0, y: 20.0 };
let p2 = p;  // Copy (not reference)
```

### 3.8 Enums

```vantor
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

// Enum with data
enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(Color),
}

let msg = Message::Move { x: 10, y: 20 };
match msg {
    Message::Move { x, y } => Console.writeLine("Move to " + x + "," + y),
    _ => {},
}
```

### 3.9 Interfaces and Traits

```vantor
interface Printable {
    fn toString() -> String;
}

interface Comparable<T> {
    fn compareTo(other: T) -> i32;
}

trait Debug {
    fn debug() -> String {
        // Default implementation
        "Debug: " + self.toString()
    }
}
```

### 3.10 Type Inference

```vantor
// Type inference with 'let'
let x = 42;              // infers i32
let name = "Alice";      // infers String
let items = List::new(); // infers List<T> (requires annotation)

// 'var' for mutable inference
var counter = 0;
counter = counter + 1;
```

---

## 4. Concurrency Model

### 4.1 Overview

VantorLang implements a **hybrid concurrency model** combining:
- **Async/Await** for I/O-bound operations
- **Structured Concurrency** with Tasks
- **Actor Model** for stateful concurrency
- **Channels** for message passing (CSP-style)

### 4.2 Async/Await

```vantor
// Async function returns a Task
pub async fn fetchUser(id: i32) -> Result<User, Error> {
    let response = await Http.get("/users/" + id)?;
    let user = Json.parse<User>(response.body)?;
    Ok(user)
}

// Multiple concurrent async operations
pub async fn fetchAllData() -> [User; 3] async {
    let user1 = await fetchUser(1);
    let user2 = await fetchUser(2);
    let user3 = await fetchUser(3);
    [user1, user2, user3]
}

// Parallel execution with 'async' block
pub async fn fetchAllParallel() -> [User; 3] {
    async {
        await fetchUser(1)
    }, async {
        await fetchUser(2)
    }, async {
        await fetchUser(3)
    }.wait_all()
}
```

### 4.3 Tasks

```vantor
// Creating and managing tasks
let task = Task.spawn(|| {
    heavyComputation()
});

// Await task result
let result = task.await;

// Task with cancellation
let task = Task.spawn(|| {
    for i in 0..1000 {
        if Task::is_cancelled() {
            break;
        }
        doWork(i);
    }
});

Task::sleep(100);
task.cancel();
```

### 4.4 Actor Model

```vantor
actor Counter {
    private count: i32 = 0;
    
    pub fn increment(&self) -> i32 {
        self.count = self.count + 1;
        self.count
    }
    
    pub fn get(&self) -> i32 {
        self.count
    }
}

let counter = Counter::new();

// Send messages asynchronously
let future = counter.increment();
let newCount = future.await;
```

### 4.5 Channels (CSP)

```vantor
// Create channel
let (sender, receiver) = Channel::<String>::new();

// Producer
Task.spawn(|| {
    sender.send("Hello".to_string());
    sender.send("World".to_string());
});

// Consumer
Task.spawn(|| {
    while let Ok(msg) = receiver.recv() {
        Console.writeLine(msg);
    }
});

// Buffered channel
let (tx, rx) = Channel::<i32>::with_buffer(100);

// Select statement
select {
    case msg <- channel1.recv() => handle(msg),
    case msg <- channel2.recv() => process(msg),
    case <-timeout => Console.writeLine("Timeout"),
}
```

### 4.6 Thread Pool

```vantor
// Execute work on thread pool
let executor = ThreadPool::new(4);

let future = executor.submit(|| {
    expensiveOperation()
});

let result = future.await;
```

### 4.7 Synchronization Primitives

```vantor
// Mutex for shared state
let mutex = Mutex::new(0);

Task.spawn(|| {
    let mut guard = mutex.lock();
    *guard = *guard + 1;
});

// RwLock for multiple readers
let rwlock = RwLock::new(Vec::new());

// Read lock
let reader = rwlock.read();
let len = reader.len();

// Write lock
let mut writer = rwlock.write();
writer.push(42);

// Atomic operations
let counter = AtomicI32::new(0);
counter.fetch_add(1);
```

---

## 5. Memory Management

### 5.1 Model: Ownership with Borrow Checker

VantorLang uses an **ownership model** similar to Rust, with:
- **Ownership**: Each value has a single owner
- **Borrowing**: References can be borrowed (mutable or immutable)
- **Lifetimes**: Compiler tracks reference lifetimes
- **Move semantics**: Ownership transfers by default

```vantor
fn main() {
    let s1 = String::from("hello");  // s1 owns the String
    let s2 = s1;                       // ownership moves to s2
    // println!("{}", s1);  // ERROR: s1 no longer valid
    
    let s3 = String::from("world");
    let len = calculate_length(&s3);  // borrow s3
    // s3 is still valid here
}

fn calculate_length(s: &String) -> usize {
    s.length()
}
```

### 5.2 Lifetimes

```vantor
// Lifetime annotations
fn longest<'a>(x: &'a String, y: &'a String) -> &'a String {
    if x.length() > y.length() { x } else { y }
}

// Struct with lifetime
struct Excerpt<'a> {
    part: &'a str,
}
```

### 5.3 Memory Regions

| Region | Description | Allocation |
|--------|-------------|------------|
| **Stack** | Fixed-size values, function locals | Compile-time |
| **Heap** | Dynamic sizes, Box<T>, collections | Runtime |
| **Static** | Constants, static variables | Compile-time |
| **Code** | Executable instructions | Load-time |

### 5.4 Box<T> and Smart Pointers

```vantor
// Box<T> - heap allocation
let boxed = Box::new(42);
let value = *boxed;  // dereference

// Rc<T> - reference counting (single-threaded)
let data = Rc::new(String::from("shared"));
let clone1 = Rc::clone(&data);
let clone2 = Rc::clone(&data);

// Arc<T> - atomic reference counting (multi-threaded)
let shared = Arc::new(Mutex::new(0));
let clone1 = Arc::clone(&shared);
```

### 5.5 Zero-Cost Abstractions

```vantor
// Iterators are zero-cost abstractions
let numbers = [1, 2, 3, 4, 5];
let sum: i32 = numbers.iter()
    .filter(|x| *x % 2 == 0)
    .map(|x| x * x)
    .sum();

// Compiles to equivalent loop
```

### 5.6 Memory Layout Optimizations

- **Struct of Arrays (SoA)** for cache locality
- **Array of Structs (AoS)** for default access patterns
- **Representation hints**:
```vantor
#[repr(C)]
struct LayoutControlled {
    // ...
}

#[repr(packed)]
struct Compact {
    a: u8,
    b: i32,
}
```

---

## 6. Compiler Architecture

### 6.1 Pipeline Overview

```
Source Code
    │
    ▼
┌─────────────────┐
│     Lexer       │  Tokenization
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Parser      │  AST Construction
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   AST/AST+     │  AST with symbols
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Type Checker    │  Type inference & checking
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   HIR          │  High-level IR
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   MIR          │  Mid-level IR (optimization)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   LLVM IR      │  Target-independent IR
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   LLVM         │  Code generation
└────────┬────────┘
         │
         ▼
   Binary/Object
```

### 6.2 Phase Descriptions

| Phase | Description |
|-------|-------------|
| **Lexer** | Tokenizes source into Token stream |
| **Parser** | Builds AST from tokens |
| **Name Resolution** | Resolves imports, scopes, symbols |
| **Type Checker** | Validates types, generics, constraints |
| **HIR** | High-level IR, desugared syntax |
| **MIR** | Mid-level IR, borrow checking, dataflow |
| **LLVM** | Generates LLVM IR |
| **Codegen** | Produces native code |

### 6.3 Backend Selection: LLVM

**Rationale:**
- Mature, well-optimized backend
- Cross-platform support
- Extensive optimization passes
- WebAssembly target support
- Active maintenance

---

## 7. Formal Grammar (EBNF)

### 7.1 Lexical Grammar

```ebnf
IDENTIFIER = letter , { letter | digit | '_' } ;
KEYWORD = 'module' | 'import' | 'pub' | 'private' | 'class' | 'struct' 
        | 'enum' | 'trait' | 'interface' | 'fn' | 'let' | 'var' | 'if'
        | 'else' | 'match' | 'for' | 'while' | 'loop' | 'return' | 'async'
        | 'await' | 'try' | 'catch' | 'throw' | 'true' | 'false' | 'null'
        | 'self' | 'super' | 'type' | 'where' | 'as' | 'is' | 'in' 
        | 'unsafe' | 'extern' | 'const' | 'mut' | 'defer' ;

INTEGER_LITERAL = [ '-' ] , digit , { digit } ;
FLOAT_LITERAL = [ '-' ] , digit , { digit } , '.' , digit , { digit } ;
STRING_LITERAL = '"' , { unicode_char - '"' } , '"' ;
CHAR_LITERAL = '\'' , ( unicode_char - '\'' ) , '\'' ;

OPERATOR = '+' | '-' | '*' | '/' | '%' | '=' | '==' | '!=' | '<' | '>'
          | '<=' | '>=' | '&&' | '||' | '!' | '&' | '|' | '^' | '~' | '<<'
          | '>>' | '=>' | '->' | '?' | '??' ;
```

### 7.2 Syntactic Grammar

```ebnf
Module = 'module' , QualifiedIdent , ';' , { Import } , { Declaration } ;

Import = 'import' , QualifiedIdent , [ 'as' , IDENTIFIER ] , ';' ;

Declaration = FunctionDecl 
             | ClassDecl 
             | StructDecl 
             | EnumDecl 
             | TraitDecl 
             | InterfaceDecl 
             | GlobalVarDecl 
             | TypeAlias ;

FunctionDecl = [ 'pub' ] , [ 'async' ] , 'fn' , IDENTIFIER , [ GenericParams ] 
             , '(' , [ ParamList ] , ')' , [ '->' , Type ] , Block ;

ClassDecl = [ 'pub' ] , 'class' , IDENTIFIER , [ GenericParams ] 
          , [ 'extends' , Type ] , [ 'implements' , TypeList ] 
          , '{' , { MemberDecl } , '}' ;

StructDecl = [ 'pub' ] , 'struct' , IDENTIFIER , [ GenericParams ] 
           , '{' , { FieldDecl } , '}' ;

EnumDecl = [ 'pub' ] , 'enum' , IDENTIFIER , [ GenericParams ] 
        , '{' , { EnumVariant } , '}' ;

TraitDecl = [ 'pub' ] , 'trait' , IDENTIFIER , [ GenericParams ] 
          , [ 'where' , WhereClause ] , '{' , { MethodSig } , '}' ;

InterfaceDecl = [ 'pub' ] , 'interface' , IDENTIFIER , [ GenericParams ] 
              , [ 'extends' , TypeList ] , '{' , { MethodSig } , '}' ;

MemberDecl = FunctionDecl | FieldDecl | ConstructorDecl ;

FieldDecl = [ 'pub' ] , [ 'mut' ] , IDENTIFIER , ':' , Type , [ '=' , Expr ] , ';' ;

ConstructorDecl = [ 'pub' ] , 'fn' , 'new' , '(' , [ ParamList ] , ')' , Block ;

MethodSig = [ 'pub' ] , [ 'async' ] , 'fn' , IDENTIFIER , [ GenericParams ] 
          , '(' , [ ParamList ] , ')' , [ '->' , Type ] , ';' ;

ParamList = Param , { ',' , Param } ;

Param = [ 'mut' ] , IDENTIFIER , ':' , Type , [ '=' , DefaultValue ] ;

GenericParams = '<' , GenericParam , { ',' , GenericParam } , '>' ;

GenericParam = IDENTIFIER , [ ':' , TypeBound ] ;

TypeBound = Type , { '+' , Type } ;

Type = PrimitiveType 
     | ArrayType 
     | NullableType 
     | GenericType 
     | ReferenceType 
     | TupleType 
 | FunctionType ;

PrimitiveType = 'bool' | 'char' | 'i8' | 'i16' | 'i32' | 'i64' | 'i128' 
              | 'u8' | 'u16' | 'u32' | 'u64' | 'u128' 
              | 'f32' | 'f64' | 'decimal' | 'void' | 'String' | 'int' | 'float' ;

NullableType = Type , '?' ;

ArrayType = Type , '[' , [ INTEGER_LITERAL ] , ']' ;

GenericType = IDENTIFIER , '<' , Type , { ',' , Type } , '>' ;

ReferenceType = '&' , [ 'mut' ] , Type ;

TupleType = '(' , Type , { ',' , Type } , ')' ;

FunctionType = 'fn' , '(' , [ TypeList ] , ')' , [ '->' , Type ] ;

Block = '{' , { Statement } , '}' ;

Statement = ExprStmt 
           | DeclStmt 
           | IfStmt 
           | MatchStmt 
           | ForStmt 
           | WhileStmt 
           | LoopStmt 
           | ReturnStmt 
           | BreakStmt 
           | ContinueStmt 
           | DeferStmt ;

ExprStmt = Expr , ';' ;

DeclStmt = 'let' , [ 'mut' ] , IDENTIFIER , [ ':' , Type ] , [ '=' , Expr ] , ';' ;

IfStmt = 'if' , Expr , Block , { 'else' , 'if' , Expr , Block } , [ 'else' , Block ] ;

MatchStmt = 'match' , Expr , '{' , MatchArm , { ',' , MatchArm } , '}' ;

MatchArm = Pattern , [ 'if' , Expr ] , '=>' , ( Expr | Block ) ;

ForStmt = 'for' , [ 'let' , Pattern ] , 'in' , Expr , Block ;

WhileStmt = 'while' , Expr , Block ;

LoopStmt = 'loop' , Block ;

ReturnStmt = 'return' , [ Expr ] , ';' ;

BreakStmt = 'break' , [ Label ] , ';' ;

ContinueStmt = 'continue' , [ Label ] , ';' ;

DeferStmt = 'defer' , Expr , ';' ;

Expr = AssignmentExpr ;

AssignmentExpr = [ 'let' , Pattern , '=' ] , ConditionalExpr 
               | ConditionalExpr , ( '=' | '+=' | '-=' | '*=' | '/=' ) , AssignmentExpr ;

ConditionalExpr = LogicalOrExpr , [ '?' , Expr , ':' , ConditionalExpr ] ;

LogicalOrExpr = LogicalAndExpr , { '||' , LogicalAndExpr } ;

LogicalAndExpr = BitOrExpr , { '&&' , BitOrExpr } ;

BitOrExpr = BitXorExpr , { '|' , BitXorExpr } ;

BitXorExpr = BitAndExpr , { '^' , BitAndExpr } ;

BitAndExpr = EqualityExpr , { '&' , EqualityExpr } ;

EqualityExpr = RelationalExpr , { ( '==' | '!=' ) , RelationalExpr } ;

RelationalExpr = ShiftExpr , [ ( '<' | '>' | '<=' | '>=' ) , ShiftExpr 
                | 'is' , Type 
                | 'as' , Type ] ;

ShiftExpr = AdditiveExpr , { ( '<<' | '>>' ) , AdditiveExpr } ;

AdditiveExpr = MultiplicativeExpr , { ( '+' | '-' ) , MultiplicativeExpr } ;

MultiplicativeExpr = UnaryExpr , { ( '*' | '/' | '%' ) , UnaryExpr } ;

UnaryExpr = ( '!' | '-' | '*' | '&' | '?' ) , UnaryExpr 
          | PostfixExpr ;

PostfixExpr = PrimaryExpr , { '.' , IDENTIFIER 
                             | '[' , Expr , ']' 
                             | '(' , [ ArgumentList ] , ')' 
                             | '?' 
                             | '!' } ;

PrimaryExpr = Literal 
            | IDENTIFIER 
            | 'self' 
            | 'super' 
            | '(' , Expr , ')' 
            | LambdaExpr 
            | TupleExpr 
            | ArrayExpr 
            | Block ;

Literal = INTEGER_LITERAL | FLOAT_LITERAL | STRING_LITERAL | CHAR_LITERAL 
        | 'true' | 'false' | 'null' ;

LambdaExpr = [ ParamList ] , '=>' , ( Expr | Block ) ;

TupleExpr = '(' , Expr , { ',' , Expr } , ')' ;

ArrayExpr = '[' , [ Expr , { ',' , Expr } ] , ']' ;

ArgumentList = Expr , { ',' , Expr } ;

TypeList = Type , { ',' , Type } ;

Pattern = IDENTIFIER 
        | '_' 
        | Literal 
        | Pattern , '|' , Pattern 
        | '[' , [ Pattern , { ',' , Pattern } ] , ']' 
        | '{' , [ FieldPattern , { ',' , FieldPattern } ] , '}' 
        | IDENTIFIER , '{' , FieldPattern , { ',' , FieldPattern } , '}' ;

FieldPattern = IDENTIFIER , [ ':' , Pattern ] ;
```

---

## 8. Build System and CLI

### 8.1 Project Configuration (vantor.toml)

```toml
[project]
name = "my-project"
version = "1.0.0"
edition = "2024"
authors = ["Developer <dev@example.com>"]
description = "A VantorLang project"

[dependencies]
core = "1.0"
http = "2.0"
json = "1.2"

[dev-dependencies]
test = "1.0"

[target]
arch = "x86_64"
os = "linux"

[build]
target = "release"
optimization = "3"
lto = true

[package]
registry = "https://vpm.vantorlang.org"

[features]
default = ["logging"]
logging = []
metrics = []
```

### 8.2 CLI Commands

```bash
# Project management
vantor new <project-name>        # Create new project
vantor init                      # Initialize in current directory
vantor update                    # Update dependencies

# Building
vantor build                     # Build project
vantor build --release          # Release build
vantor build --target <target>  # Cross-compile
vantor watch                    # Watch mode

# Running
vantor run                       # Build and run
vantor run --args <args>         # Pass arguments

# Testing
vantor test                      # Run tests
vantor test --verbose           # Verbose output
vantor test --coverage          # Coverage report

# Package management
vanto search <query>             # Search packages
vanto install <package>          # Install package
vanto remove <package>          # Remove package
vanto publish                    # Publish package

# Tools
vantor fmt                       # Format code
vantor check                     # Type check only
vantor doc                       # Generate documentation
vantor lint                      # Run linter
vantor benchmark                 # Run benchmarks
```

### 8.3 Directory Structure

```
my-project/
├── src/
│   ├── main.vnt
│   ├── lib.vnt
│   ├── module/
│   │   ├── handler.vnt
│   │   └── types.vnt
│   └── generated/
├── tests/
│   ├── integration/
│   └── unit/
├── resources/
│   ├── assets/
│   └── config/
├── docs/
├── examples/
├── build/
│   ├── debug/
│   └── release/
├── target/
│   └── debug/
├── vantor.toml
├── README.md
├── LICENSE
└── .gitignore
```

---

## 9. Package Management

### 9.1 VPM (Vantor Package Manager)

```bash
# Search for packages
vpm search json
vpm search "parser~2.0"

# Install packages
vpm install json
vpm install json@2.1.0
vpm install "json>=2.0,<3.0"

# Publish package
vpm publish
vpm publish --access public
```

### 9.2 Package Structure

```
vantor-json/
├── vantor.toml
├── src/
│   ├── lib.vnt
│   ├── parser.vnt
│   └── encoder.vnt
├── tests/
├── docs/
└── LICENSE
```

### 9.3 Package Manifest

```toml
[package]
name = "vantor-json"
version = "2.1.0"
edition = "2024"
authors = ["Vantor Team <team@vantorlang.org>"]
description = "JSON serialization library"
license = "MIT"
repository = "https://github.com/vantorlang/json"
documentation = "https://docs.vantorlang.org/json"
keywords = ["json", "serialization", "parser"]
categories = ["encoding", "parser"]

[dependencies]
core = ">=1.0"

[dev-dependencies]
vantor-test = "1.0"

[features]
default = ["std"]
std = []
no-alloc = []

[export]
types = ["JsonValue", "JsonParser"]
```

---

## 10. Testing Framework

### 10.1 Test Attributes

```vantor
// Unit test
@test
fn test_addition() {
    assert 2 + 2 == 4;
}

// Test with setup/teardown
@test(before = setup_db, after = cleanup_db)
fn test_database() {
    // ...
}

// Test with custom name
@test("Test user creation")
fn create_user_test() {
    let user = User::new("Alice", 30);
    assert user.name == "Alice";
}

// Ignore test
@ignore
fn test_unimplemented() {
    // ...
}

// Test with expected panic
@test(should_panic = "IndexOutOfBounds")
fn test_out_of_bounds() {
    let arr = [1, 2, 3];
    let _ = arr[10];
}
```

### 10.2 Test Organization

```vantor
mod tests {
    use super::*;
    
    @test
    fn test_private_method() {
        // Can test private members within same module
        let obj = MyClass::new();
        assert obj.internal_method() == expected;
    }
}

mod integration_tests {
    @test
    async fn test_api_endpoint() {
        let response = await api::get("/health");
        assert response.status == 200;
    }
}
```

### 10.3 Assertions

```vantor
// Basic assertions
assert condition;
assert condition, "Custom message";

// Equality
assert_eq!(a, b);
assert_ne!(a, b);

// Type checks
assert_type!(value, i32);
assert_impl!(value, Printable);

// Collections
assert_contains!(list, item);
assert_empty!(collection);
assert_len!(collection, 3);

// Errors
assert_ok!(result);
assert_err!(result);
```

### 10.4 Test Runner

```bash
# Run all tests
vantor test

# Run specific test
vantor test test_addition

# Run tests matching pattern
vantor test --filter "user*"

# Run with coverage
vantor test --coverage

# Run benchmarks
vantor test --bench
```

---

## 11. Code Examples

### 11.1 Hello World

```vantor
module hello;

import core.io.Console;

pub fn main() -> void {
    Console.writeLine("Hello, World!");
}
```

### 11.2 Class with Inheritance

```vantor
module shapes;

interface Shape {
    fn area() -> f64;
    fn perimeter() -> f64;
}

abstract class AbstractShape {
    pub abstract fn area() -> f64;
    pub abstract fn perimeter() -> f64;
    
    pub fn describe(&self) -> void {
        Console.writeLine("Area: " + self.area().toString());
    }
}

class Rectangle : AbstractShape {
    private width: f64;
    private height: f64;
    
    pub fn new(width: f64, height: f64) -> Rectangle {
        Rectangle { width, height }
    }
    
    pub override fn area() -> f64 {
        self.width * self.height
    }
    
    pub override fn perimeter() -> f64 {
        2.0 * (self.width + self.height)
    }
}

class Circle : AbstractShape {
    private radius: f64;
    
    pub fn new(radius: f64) -> Circle {
        Circle { radius }
    }
    
    pub override fn area() -> f64 {
        3.14159 * self.radius * self.radius
    }
    
    pub override fn perimeter() -> f64 {
        2.0 * 3.14159 * self.radius
    }
}
```

### 11.3 Async HTTP Server

```vantor
module http_server;

import core.net.*;
import core.io.Console;
import core.collections.*;

pub async fn main() -> void {
    let server = HttpServer::new("127.0.0.1:8080");
    
    server.route("/api/users", handle_users);
    server.route("/api/posts", handle_posts);
    server.route("/", handle_static);
    
    Console.writeLine("Server starting on http://127.0.0.1:8080");
    await server.listen();
}

async fn handle_users(req: HttpRequest) -> HttpResponse {
    let users = database::get_all_users()?;
    
    HttpResponse::ok()
        .json(users)
}

async fn handle_posts(req: HttpRequest) -> HttpResponse {
    let user_id = req.query_params().get("user_id")?;
    let posts = database::get_posts(user_id)?;
    
    HttpResponse::ok()
        .json(posts)
}
```

### 11.4 Actor-Based State

```vantor
module bank;

actor BankAccount {
    private balance: f64 = 0.0;
    
    pub async fn deposit(amount: f64) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        self.balance = self.balance + amount;
        Ok(())
    }
    
    pub async fn withdraw(amount: f64) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if amount > self.balance {
            return Err(Error::InsufficientFunds);
        }
        self.balance = self.balance - amount;
        Ok(())
    }
    
    pub async fn get_balance() -> f64 {
        self.balance
    }
}

async fn main() -> void {
    let account = BankAccount::new();
    
    await account.deposit(1000.0);
    await account.withdraw(250.0);
    
    let balance = await account.get_balance();
    Console.writeLine("Balance: " + balance.toString());
}
```

### 11.5 Error Handling

```vantor
module error_handling;

import core.io.Console;

enum AppError {
    InvalidInput(String),
    NotFound(String),
    PermissionDenied,
    IoError(IOException),
}

impl Error for AppError {
    fn message(&self) -> String {
        match self {
            InvalidInput(msg) => "Invalid input: " + msg,
            NotFound(what) => "Not found: " + what,
            PermissionDenied => "Permission denied",
            IoError(e) => "IO error: " + e.message(),
        }
    }
    
    fn code(&self) -> i32 {
        match self {
            InvalidInput(_) => 400,
            NotFound(_) => 404,
            PermissionDenied => 403,
            IoError(_) => 500,
        }
    }
}

fn process_user_input(input: String) -> Result<User, AppError> {
    if input.is_empty() {
        return Err(AppError::InvalidInput("Empty input".to_string()));
    }
    
    let user = find_user(input)?;
    match user {
        Some(u) => Ok(u),
        None => Err(AppError::NotFound(input)),
    }
}
```

---

## 12. Technical Comparison

### 12.1 Feature Comparison Matrix

| Feature | VantorLang | Java | C# | Rust |
|---------|------------|------|-----|------|
| **Type System** | | | | |
| Static typing | ✅ | ✅ | ✅ | ✅ |
| Type inference | ✅ | ✅ (var) | ✅ | ✅ |
| Null safety | ✅ | ❌ | ✅ (nullable) | ✅ |
| Generics | ✅ | ✅ | ✅ | ✅ |
| Traits/Interfaces | ✅ | ✅ | ✅ | ✅ |
| Algebraic Data Types | ✅ | ❌ | ✅ (records) | ✅ |
| **Memory** | | | | |
| Ownership model | ✅ | GC | GC | ✅ |
| Borrow checking | ✅ | ❌ | ❌ | ✅ |
| RAII | ✅ | ❌ | ✅ | ✅ |
| **Concurrency** | | | | |
| Async/Await | ✅ | ✅ (virtual threads) | ✅ | ✅ |
| Actors | ✅ | ✅ (Akka) | ✅ (CAF) | ✅ |
| Channels | ✅ | ❌ | ✅ | ✅ |
| Green threads | ✅ | ✅ | ❌ | ❌ |
| **Performance** | | | | |
| Native compilation | ✅ | JIT/AOT | JIT/AOT | ✅ |
| Zero-cost abstractions | ✅ | Limited | Limited | ✅ |
| Memory safety | ✅ | ✅ | ✅ | ✅ |
| **Ecosystem** | | | | |
| Package manager | VPM | Maven | NuGet | Cargo |
| Build tool | Built-in | Gradle/Maven | MSBuild | Cargo |
| IDE support | Planned | Excellent | Excellent | Good |

### 12.2 Performance Benchmarks (Projected)

| Benchmark | VantorLang | Java | C# | Rust |
|-----------|------------|------|-----|------|
| **Raytrace** | 1.0x | 1.2x | 1.1x | 0.95x |
| **JSON Parse** | 1.0x | 1.5x | 1.3x | 0.9x |
| **Binary Trees** | 1.0x | 1.8x | 1.6x | 0.85x |
| **Fannkuch** | 1.0x | 1.3x | 1.2x | 0.95x |
| **Startup Time** | 1.0x | 3.0x | 2.5x | 0.3x |

*Note: Projected values based on architectural analysis. Actual performance TBD.*

### 12.3 Design Decisions Justification

**Why not pure GC?**
- GC adds pause times unacceptable for real-time systems
- Ownership model enables more predictable memory behavior
- Better for systems programming where determinism matters

**Why not pure borrow checker?**
- Steep learning curve for developers from GC languages
- Hybrid approach: ownership semantics with optional safe defaults
- Gradual learning path: start simple, adopt advanced patterns as needed

**Why async/await + actors?**
- Async/await: Industry standard for I/O-bound operations
- Actors: Natural fit for concurrent state management
- Channels: Simple, proven model (Go, Rust)

---

## 13. Compiler Implementation (Rust)

### 13.1 Project Structure

```
vantor/
├── Cargo.toml
├── vantor-cli/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── build.rs
│   │   │   ├── run.rs
│   │   │   ├── test.rs
│   │   │   └── fmt.rs
│   │   └── opts.rs
├── vantor-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       └── span.rs
├── vantor-lexer/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── tokenizer.rs
│       └── tokens.rs
├── vantor-parser/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── grammar.rs
│       └── ast/
├── vantor-analysis/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── name_res.rs
│       ├── typeck.rs
│       └── hir.rs
├── vantor-codegen/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── llvm.rs
│       └── target.rs
└── vantor-utils/
    └── src/
        └── lib.rs
```

### 13.2 Cargo Workspace

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "vantor-cli",
    "vantor-core",
    "vantor-lexer",
    "vantor-parser",
    "vantor-analysis",
    "vantor-codegen",
    "vantor-utils",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "MIT"
authors = ["VantorLang Team"]

[workspace.dependencies]
llvm-sys = "16.0"
logos = "0.14"
```

### 13.3 Token Definition

```rust
// vantor-lexer/src/tokens.rs

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
pub enum Token {
    // Keywords
    #[token("module")]
    Module,
    #[token("import")]
    Import,
    #[token("pub")]
    Pub,
    #[token("private")]
    Private,
    #[token("class")]
    Class,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("trait")]
    Trait,
    #[token("interface")]
    Interface,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("var")]
    Var,
    #[token("mut")]
    Mut,
    #[token("const")]
    Const,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("loop")]
    Loop,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("defer")]
    Defer,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("throw")]
    Throw,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("self")]
    Self_,
    #[token("super")]
    Super,
    #[token("type")]
    Type,
    #[token("where")]
    Where,
    #[token("as")]
    As,
    #[token("is")]
    Is,
    #[token("in")]
    In,
    #[token("unsafe")]
    Unsafe,
    #[token("extern")]
    Extern,
    
    // Operators
    #[token("=>")]
    FatArrow,
    #[token("->")]
    RArrow,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("??")]
    QuestionQuestion,
    #[token("::")]
    PathSep,
    
    // Punctuation
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("!")]
    Bang,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("?")]
    Question,
    
    // Literals
    #[regex(r"[0-9]+", priority = 2)]
    IntLit,
    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLit,
    #[regex(r#""[^"\\]*(?:\\.[^"\\]*)*""#)]
    StringLit,
    #[regex(r"'[^'\\]'")]
    CharLit,
    
    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
    
    // Special
    #[regex(r"//.*")]
    Comment,
    #[regex(r"/\*[\s\S]*?\*/")]
    BlockComment,
    
    Error,
    Eof,
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(self, Token::Module | Token::Import | Token::Class | ...)
    }
    
    pub fn is_literal(&self) -> bool {
        matches!(self, Token::IntLit | Token::FloatLit | Token::StringLit | ...)
    }
}
```

### 13.4 Lexer Implementation

```rust
// vantor-lexer/src/tokenizer.rs

use crate::tokens::Token;
use logos::Logos;
use std::iter::Pe std::str::ekable;
useCharIndices;

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    current: Option<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices().peekable();
        let current = chars.next().map(|(_, c)| c);
        
        Lexer {
            source,
            chars,
            current,
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        
        let mut lexer = Token::lexer(self.source);
        
        while let Some(token) = lexer.next() {
            let span = Span::new(lexer.span());
            
            match token {
                Ok(t) if !t.is_whitespace() => {
                    tokens.push(SpannedToken { token: t, span });
                }
                Err(_) => {
                    // Handle lexical error
                    let error = LexicalError::new(
                        format!("Unexpected character '{}'", lexer.slice()),
                        span,
                    );
                    tokens.push(SpannedToken {
                        token: Token::Error,
                        span,
                    });
                }
                _ => {}
            }
        }
        
        tokens
    }
    
    pub fn position(&self) -> Span {
        Span::new(self.position..self.position + 1)
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(range: std::ops::Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
            line: 0, // Would need to track
            column: 0,
        }
    }
    
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}
```

### 13.5 AST Definitions

```rust
// vantor-parser/src/ast/mod.rs

use crate::tokens::{Span, SpannedToken};

#[derive(Debug, Clone)]
pub enum AstNode {
    Module(Module),
    Import(Import),
    Function(Function),
    Class(Class),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Interface(Interface),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Path,
    pub imports: Vec<Import>,
    pub declarations: Vec<Decl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub segments: Vec<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub extends: Option<Type>,
    pub implements: Vec<Type>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Function(Function),
    Class(Class),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Interface(Interface),
    GlobalVar(GlobalVar),
    TypeAlias(TypeAlias),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(ExprStmt),
    Decl(DeclStmt),
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Loop(LoopStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Defer(DeferStmt),
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DeclStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    FieldAccess(FieldAccessExpr),
    MethodCall(MethodCallExpr),
    Literal(Literal),
    Ident(Ident),
    Lambda(Lambda),
    Block(Box<Block>),
    If(IfExpr),
    Match(MatchExpr),
    Try(TryExpr),
    Await(AwaitExpr),
    // ... more variants
}

#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Int(i64, IntSuffix),
    Float(f64, FloatSuffix),
    String(String),
    Char(char),
    Array(Vec<Expression>),
    Tuple(Vec<Expression>),
}

// Type definitions would continue similarly...
```

### 13.6 Parser Skeleton

```rust
// vantor-parser/src/parser.rs

use crate::ast::*;
use crate::tokens::{Token, SpannedToken, Span};
use crate::error::{ParseError, ParseResult};

pub struct Parser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [SpannedToken]) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }
    
    pub fn parse(&mut self) -> ParseResult<Module> {
        self.parse_module()
    }
    
    fn current(&self) -> &SpannedToken {
        self.tokens.get(self.pos).unwrap_or(&SpannedToken {
            token: Token::Eof,
            span: Span::default(),
        })
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }
    
    fn expect(&mut self, expected: Token) -> ParseResult<SpannedToken> {
        let current = self.current().clone();
        if std::mem::discriminant(&current.token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(current)
        } else {
            Err(ParseError::unexpected_token(current, expected))
        }
    }
    
    fn parse_module(&mut self) -> ParseResult<Module> {
        self.expect(Token::Module)?;
        let name = self.parse_path()?;
        self.expect(Token::Semi)?;
        
        let mut imports = Vec::new();
        while self.check(Token::Import) {
            imports.push(self.parse_import()?);
        }
        
        let mut declarations = Vec::new();
        while !self.check(Token::Eof) {
            declarations.push(self.parse_declaration()?);
        }
        
        Ok(Module {
            name,
            imports,
            declarations,
            span: Span::default(), // Would compute from children
        })
    }
    
    fn parse_import(&mut self) -> ParseResult<Import> {
        self.expect(Token::Import)?;
        let path = self.parse_path()?;
        let alias = if self.check(Token::As) {
            self.advance();
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.expect(Token::Semi)?;
        
        Ok(Import {
            path,
            alias,
            span: Span::default(),
        })
    }
    
    fn parse_declaration(&mut self) -> ParseResult<Decl> {
        let attrs = self.parse_attributes()?;
        
        let visibility = if self.check(Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        
        match self.current().token {
            Token::Fn => Ok(Decl::Function(self.parse_function(visibility, attrs)?)),
            Token::Class => Ok(Decl::Class(self.parse_class(visibility, attrs)?)),
            Token::Struct => Ok(Decl::Struct(self.parse_struct(visibility, attrs)?)),
            Token::Enum => Ok(Decl::Enum(self.parse_enum(visibility, attrs)?)),
            Token::Trait => Ok(Decl::Trait(self.parse_trait(visibility, attrs)?)),
            Token::Type => Ok(Decl::TypeAlias(self.parse_type_alias(visibility)?)),
            Token::Const | Token::Let => Ok(Decl::GlobalVar(self.parse_global_var(visibility)?)),
            _ => Err(ParseError::unexpected_token(self.current().clone())),
        }
    }
    
    fn parse_function(&mut self, visibility: Visibility, attrs: Vec<Attribute>) -> ParseResult<Function> {
        let is_async = if self.check(Token::Async) {
            self.advance();
            true
        } else {
            false
        };
        
        self.expect(Token::Fn)?;
        let name = self.parse_ident()?;
        
        let type_params = self.parse_generic_params()?;
        
        self.expect(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(Token::RParen)?;
        
        let return_type = if self.check(Token::RArrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = if self.check(Token::LBrace) {
            Some(self.parse_block()?)
        } else {
            self.expect(Token::Semi)?;
            None
        };
        
        Ok(Function {
            attrs,
            visibility,
            is_async,
            name,
            type_params,
            params,
            return_type,
            body,
            span: Span::default(),
        })
    }
    
    // ... additional parsing methods
}
```

### 13.7 Type Checker (Simplified)

```rust
// vantor-analysis/src/typeck.rs

use crate::ast::*;
use crate::hir::*;
use crate::error::{TypeError, TypeResult};
use std::collections::{HashMap, HashSet};

pub struct TypeChecker {
    scopes: Vec<Scope>,
    current_function: Option<FunctionId>,
    errors: Vec<TypeError>,
    type_defs: HashMap<Path, TypeDef>,
    substitutions: HashMap<TypeParam, Type>,
}

struct Scope {
    bindings: HashMap<Ident, Binding>,
    parent: Option<usize>,
}

pub struct TypeChecker {
    scopes: Vec<Scope>,
    type_defs: HashMap<Path, TypeDef>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(Scope {
            bindings: HashMap::new(),
            parent: None,
        });
        
        TypeChecker {
            scopes,
            type_defs: HashMap::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn check_module(&mut self, module: &Module) -> TypeResult<HirModule> {
        // First pass: register all types
        self.register_types(module)?;
        
        // Second pass: check declarations
        for decl in &module.declarations {
            self.check_declaration(decl)?;
        }
        
        if self.errors.is_empty() {
            Ok(HirModule { /* ... */ })
        } else {
            Err(TypeError::Multiple(self.errors.clone()))
        }
    }
    
    fn register_types(&mut self, module: &Module) -> TypeResult<()> {
        for decl in &module.declarations {
            match decl {
                Decl::Struct(s) => {
                    let type_def = TypeDef::Struct(TypeDefStruct {
                        name: s.name.clone(),
                        type_params: s.type_params.clone(),
                        fields: s.fields.iter().map(|f| (f.name.clone(), f.ty.clone())).collect(),
                    });
                    self.type_defs.insert(Path::from_ident(&s.name), type_def);
                }
                Decl::Enum(e) => {
                    let type_def = TypeDef::Enum(TypeDefEnum {
                        name: e.name.clone(),
                        type_params: e.type_params.clone(),
                        variants: e.variants.iter().map(|v| (v.name.clone(), v.data.clone())).collect(),
                    });
                    self.type_defs.insert(Path::from_ident(&e.name), type_def);
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn check_declaration(&mut self, decl: &Decl) -> TypeResult<Ty> {
        match decl {
            Decl::Function(f) => {
                self.check_function(f)?;
                Ok(Ty::Void)
            }
            Decl::Class(c) => {
                self.check_class(c)?;
                Ok(Ty::Void)
            }
            // ... other declarations
        }
    }
    
    fn check_function(&mut self, func: &Function) -> TypeResult<Ty> {
        self.push_scope();
        
        // Register parameters
        for param in &func.params {
            let param_ty = self.resolve_type(&param.ty)?;
            self.declare(param.name.clone(), param_ty);
        }
        
        // Check return type
        let return_ty = match &func.return_type {
            Some(ty) => self.resolve_type(ty)?,
            None => Ty::Void,
        };
        
        // Check body
        if let Some(body) = &func.body {
            let body_ty = self.check_block(body)?;
            self.unify(body_ty, return_ty.clone())?;
        }
        
        self.pop_scope();
        Ok(return_ty)
    }
    
    fn check_expression(&mut self, expr: &Expression) -> TypeResult<Ty> {
        match expr {
            Expression::Literal(lit) => self.check_literal(lit),
            Expression::Ident(id) => self.lookup_type(id),
            Expression::Binary(bin) => self.check_binary(bin),
            Expression::Unary(un) => self.check_unary(un),
            Expression::Call(call) => self.check_call(call),
            Expression::Lambda(lambda) => self.check_lambda(lambda),
            // ... other cases
        }
    }
    
    fn check_binary(&mut self, bin: &BinaryExpr) -> TypeResult<Ty> {
        let lhs_ty = self.check_expression(&bin.lhs)?;
        let rhs_ty = self.check_expression(&bin.rhs)?;
        
        match bin.op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                // Numeric operations require numeric types
                if lhs_ty.is_numeric() && rhs_ty.is_numeric() {
                    // Return the wider type
                    Ok(widest_numeric(&lhs_ty, &rhs_ty))
                } else {
                    Err(TypeError::type_mismatch(
                        format!("Cannot perform arithmetic on {} and {}", lhs_ty, rhs_ty),
                    ))
                }
            }
            BinOp::Eq | BinOp::Neq => {
                self.unify(lhs_ty.clone(), rhs_ty.clone())?;
                Ok(Ty::Bool)
            }
            BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => {
                self.unify(lhs_ty.clone(), rhs_ty.clone())?;
                Ok(Ty::Bool)
            }
            BinOp::And | BinOp::Or => {
                self.unify(lhs_ty.clone(), Ty::Bool)?;
                self.unify(rhs_ty.clone(), Ty::Bool)?;
                Ok(Ty::Bool)
            }
        }
    }
    
    fn check_call(&mut self, call: &CallExpr) -> TypeResult<Ty> {
        let callee_ty = self.check_expression(&call.callee)?;
        
        match callee_ty {
            Ty::Function(params, ret) => {
                // Unify call arguments with parameter types
                for (arg, param) in call.args.iter().zip(params.iter()) {
                    let arg_ty = self.check_expression(arg)?;
                    self.unify(arg_ty, param.clone())?;
                }
                Ok(*ret)
            }
            Ty::Builtin(name) => {
                // Handle builtin functions
                Ok(self.check_builtin_call(name, call)?)
            }
            _ => Err(TypeError::not_callable(callee_ty)),
        }
    }
    
    fn resolve_type(&self, ty: &Type) -> TypeResult<Ty> {
        match ty {
            Type::Path(path) => {
                self.type_defs
                    .get(path)
                    .map(|def| Ty::Def(path.clone()))
                    .ok_or_else(|| TypeError::unknown_type(path.to_string()))
            }
            Type::Nullable(t) => {
                let inner = self.resolve_type(t)?;
                Ok(Ty::Option(Box::new(inner)))
            }
            Type::Array(t, size) => {
                let inner = self.resolve_type(t)?;
                Ok(Ty::Array(Box::new(inner), *size))
            }
            Type::Function(params, ret) => {
                let params = params.iter().map(|p| self.resolve_type(p)).collect::<TypeResult<_>>()?;
                let ret = self.resolve_type(ret)?;
                Ok(Ty::Function(params, Box::new(ret)))
            }
            // ... other cases
        }
    }
    
    fn unify(&mut self, ty1: Ty, ty2: Ty) -> TypeResult<()> {
        // Handle type unification with subtyping
        if ty1 == ty2 {
            return Ok(());
        }
        
        // Handle nullability
        match (&ty1, &ty2) {
            (Ty::Option(a), Ty::Option(b)) => self.unify(*a.clone(), *b.clone()),
            (Ty::Option(_), Ty::Null) => Ok(()),
            (Ty::Null, Ty::Option(_)) => Ok(()),
            // ... more cases
            _ => Err(TypeError::type_mismatch(format!("{} vs {}", ty1, ty2))),
        }
    }
}
```

---

## 14. Development Roadmap

### Phase 1: MVP (Months 1-6)

| Week | Goal |
|------|------|
| 1-2 | Project setup, lexer implementation |
| 3-4 | Basic parser, AST generation |
| 5-6 | Name resolution, basic type checker |
| 7-8 | Code generation (simple functions) |
| 9-10 | CLI implementation |
| 11-12 | Testing infrastructure |
| 13-14 | Basic stdlib (core.io) |
| 15-16 | First working "Hello World" |
| 17-20 | Debug builds, error handling |
| 21-24 | Documentation, release v0.1 |

**Deliverables:**
- Working compiler for basic subset
- Hello World execution
- Basic error messages

### Phase 2: Type System Complete (Months 7-12)

| Week | Goal |
|------|------|
| 25-28 | Generics implementation |
| 29-32 | Nullable types, Option<T> |
| 33-36 | Trait/interface system |
| 37-40 | Enums with data |
| 41-44 | Pattern matching |
| 45-48 | Type inference improvements |

**Deliverables:**
- Full type system
- Generics with constraints
- Algebraic data types

### Phase 3: Concurrency (Months 13-18)

| Week | Goal |
|------|------|
| 49-52 | Async/await implementation |
| 53-56 | Task runtime |
| 57-60 | Channel implementation |
| 61-64 | Actor model |
| 65-68 | Thread pool |
| 69-72 | Async stdlib (net, io) |

**Deliverables:**
- Full async runtime
- Concurrent HTTP server example

### Phase 4: Optimizations (Months 19-24)

| Week | Goal |
|------|------|
| 73-76 | LLVM optimization passes |
| 77-80 | Inlining, constant propagation |
| 81-84 | Escape analysis |
| 85-88 | SIMD intrinsics |
| 89-92 | Link-time optimization |
| 93-96 | Performance benchmarking |

**Deliverables:**
- Optimized release builds
- Performance parity targets

### Phase 5: Ecosystem (Months 25-30)

| Week | Goal |
|------|------|
| 97-100 | Package manager (VPM) |
| 101-104 | Standard library expansion |
| 105-108 | Database drivers |
| 109-112 | HTTP frameworks |
| 113-116 | Serialization (JSON, XML) |
| 117-120 | Testing frameworks |

**Deliverables:**
- Functional package ecosystem
- Production-ready stdlib

### Phase 6: IDE Support (Months 31-36)

| Week | Goal |
|------|------|
| 121-124 | Language Server Protocol |
| 125-128 | VSCode extension |
| 129-132 | IntelliJ plugin |
| 133-136 | Debugger integration |
| 137-140 | Refactoring tools |
| 141-144 | Documentation generation |

**Deliverables:**
- Full IDE support
- Production release v1.0

---

## Appendix A: Reserved Words

```
as, async, await, break, catch, channel, class, const, continue,
crate, defer, do, else, enum, error, extern, false, final, fn,
for, if, impl, import, in, inline, interface, is, let, loop,
match, mod, module, mut, namespace, native, new, null, override,
priv, pub, readonly, ref, return, self, Self, static, struct,
super, switch, trait, true, try, type, typeof, unsafe, use,
var, virtual, void, where, while, yield
```

---

## Appendix B: Built-in Types

```
bool, char, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128,
f32, f64, decimal, string, String, Array<T>, List<T>, Map<K,V>,
Set<T>, Option<T>, Result<T,E>, Box<T>, Rc<T>, Arc<T>, Mutex<T>
```

---

## Appendix C: Standard Library Modules

```
core          - Core types and utilities
core.io       - Input/output (Console, File, Buffer)
core.net      - Networking (Http, Tcp, Udp, WebSocket)
core.threads  - Threading primitives
core.sync     - Synchronization (Mutex, RwLock, Channel)
core.time     - Time and date
core.math     - Mathematical functions
core.mem      - Memory operations
collections   - Collections (List, Map, Set, Queue, Stack)
json          - JSON serialization
xml           - XML parsing
regex         - Regular expressions
datetime      - Date/time handling
uuid          - UUID generation
crypto        - Cryptographic functions
logging       - Logging framework
testing       - Testing utilities
```

---

## Appendix D: Unique Advanced Features

### D.1 Linear Types (Ownership without Borrow Checker)

Unlike Rust's borrow checker, VantorLang supports **linear types** where values must be used exactly once:

```vantor
// Linear type - must be consumed
resource FileHandle {
    pub fn read() -> String;
    pub fn close() -> void;
}

// Ensures file is always closed - no leaked resources
fn processFile(path: String) -> void {
    let file = FileHandle::open(path)?;  // Linear - must use
    let content = file.read();
    file.close();  // Must call - compiler enforces
}

// Alternative: using linear in expression
fn processFile2(path: String) -> void {
    FileHandle::open(path)
        .read()
        .close();  // Ensures cleanup
}
```

### D.2 Algebraic Effects (Like Koka/OCaml)

First-class support for algebraic effects and handlers:

```vantor
// Define effect
effect Io {
    effect readFile(path: String) -> String;
    effect writeFile(path: String, content: String) -> void;
    effect print(msg: String) -> void;
}

// Function that performs effects
fn readAndProcess(path: String) -> String!Io {
    let content = perform Io.readFile(path);
    perform Io.print("Read " + content.length() + " bytes");
    content.toUpperCase()
}

// Handler that provides implementations
fn main() -> void {
    let result = handle readAndProcess("test.txt") {
        Io.readFile(path) => resume(fs.read(path));
        Io.print(msg) => {
            Console.writeLine(msg);
            resume(());
        }
    };
}
```

### D.3 Compile-Time Code Generation (Macros)

Powerful macro system that generates code at compile time:

```vantor
// Generate boilerplate code
macro derive(serializer, T) {
    impl Serializable for T {
        fn serialize(&self) -> Bytes {
            let mut buf = Bytes::new();
            $(for field in T.fields) {
                buf.write(self.$(field.name).serialize());
            }
            buf
        }
    }
}

#[derive(serializer)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Auto-generates: serialize(), deserialize(), hash(), equals()
```

### D.4 Dependent Types Lite

Types that can depend on values for compile-time verification:

```vantor
// Sized arrays - compile-time bounds checking
fn sort<T>(arr: [T; n], n: usize) -> [T; n] where T: Comparable {
    // Compiler knows exact size at compile time
}

// Dependent function types
fn safeIndex<T>(arr: [T], i: usize) -> T 
    requires i < arr.length  // Compile-time assertion
{
    arr[i]  // No bounds check needed - proven safe
}

// Type-level integers
type Nat = 0 | (Nat + 1);
type Fin<n> = { i: i32 | 0 <= i && i < n };
```

### D.5 Built-in Contract System (Design by Contract)

Language-level support for preconditions, postconditions, and invariants:

```vantor
class BankAccount {
    private balance: f64;
    
    pub fn deposit(amount: f64) -> void
        requires amount > 0, "Deposit must be positive"
        ensures balance == old(balance) + amount
    {
        self.balance = self.balance + amount;
    }
    
    pub fn withdraw(amount: f64) -> void
        requires amount > 0
        requires amount <= balance, "Insufficient funds"
        ensures balance == old(balance) - amount
    {
        self.balance = self.balance - amount;
    }
    
    invariant balance >= 0, "Balance cannot be negative"
}

// Contracts can be checked at runtime in debug mode
// Or verified statically in release mode with formal methods
```

### D.6 Zero-Cost Async Runtime (Better than Go)

Green threads com scheduling cooperativo extremamente eficiente:

```vantor
// Spawn 1 million coroutines with minimal overhead
pub async fn main() -> void {
    let tasks = Vec::new();
    
    for i in 0..1_000_000 {
        tasks.push(spawn processItem(i));
    }
    
    // Wait all - efficient event loop
    await all(tasks);
}

// Async streams - infinite iterators
async fn watchFiles() -> Stream<FileEvent> {
    for event in fs.watch("/var/log") {
        yield event;
    }
}

// Select with deadlines
select {
    case result <- asyncTask() => handle(result),
    case <- timeout(5.seconds()) => {
        Console.writeLine("Task timed out");
    }
}
```

### D.7 Hot Code Reloading (Live Coding)

Update code without restarting:

```vantor
// Server with hot reload
#[hot_reload]
pub async fn handleRequest(req: Request) -> Response {
    let cached = cache.get(req.path);
    match cached {
        Some(data) => Response::cached(data),
        None => {
            let result = await db.query(req.path);
            cache.set(req.path, result);
            Response::ok(result)
        }
    }
}

// In development: changes are applied instantly
// Without losing state (for stateful services)
```

### D.8 Built-in Property-Based Testing

```vantor
// Automatically generate hundreds of test cases
#[property]
fn test_sort_preserves_length() {
    let arr = any::<[i32]>();
    let sorted = sort(arr);
    assert sorted.length() == arr.length();
}

#[property]
fn test_sort_is_ordered() {
    let arr = any::<[i32]>();
    let sorted = sort(arr);
    for i in 0..sorted.length() - 1 {
        assert sorted[i] <= sorted[i + 1];
    }
}

// Fuzzing integration
#[fuzz]
fn test_json_parser(data: Vec<u8>) {
    let result = Json.parse(data);
    // Test that parser doesn't crash on arbitrary input
}
```

### D.9 Quantum-Safe Cryptography (First-Class)

Built-in post-quantum cryptographic algorithms:

```vantor
use crypto.kyber;
use crypto.x25519;

// Post-quantum key exchange
fn generateKeyPair() -> (KyberPublicKey, KyberSecretKey) {
    let (pk, sk) = kyber.generate();
    (pk, sk)
}

// Hybrid classical + quantum-safe
fn hybridEncrypt(msg: Bytes, pk: KyberPublicKey) -> HybridCiphertext {
    let classical = x25519.encrypt(msg, pk.classical());
    let quantum = kyber.encrypt(msg, pk);
    HybridCiphertext { classical, quantum }
}

// Hash-based signatures (for blockchain)
use crypto.sphincs;

fn sign(message: Bytes, key: sphincs.PrivateKey) -> sphincs.Signature {
    sphincs.sign(message, key)
}
```

### D.10 WebGPU Compute Shaders (GPU Programming)

First-class GPU programming without external languages:

```vantor
// Direct GPU compute kernels
compute vecAdd(a: &[f32], b: &[f32], result: &mut [f32], n: usize) {
    let idx = globalId.x;
    if idx < n {
        result[idx] = a[idx] + b[idx];
    }
}

fn main() {
    let gpu = GPU::new();
    
    let a = gpu.alloc::<f32>(1024 * 1024);
    let b = gpu.alloc::<f32>(1024 * 1024);
    let result = gpu.alloc::<f32>(1024 * 1024);
    
    // Launch GPU kernel
    gpu.launch(vecAdd, [1024, 1024], a, b, result, 1024 * 1024);
    
    // Read back result
    let sum = result.read();
}
```

### D.11 Multi-Target Compilation

Compile to multiple targets simultaneously:

```vantor
// Single source, multiple targets
#[target("x86_64-unknown-linux-gnu")]
#[target("aarch64-unknown-linux-gnu")]
#[target("x86_64-pc-windows-msvc")]
#[target("wasm32-unknown-unknown")]
pub fn process(data: Data) -> Result {
    // Platform-specific code via traits
}

// Conditional compilation
#[cfg(target_os = "linux")]
fn platformInit() { /* Linux-specific */ }

#[cfg(target_os = "windows")]
fn platformInit() { /* Windows-specific */ }
```

### D.12 Formal Verification Integration

```vantor
// Prove properties about your code
prove fn addCommutative(a: i32, b: i32) -> bool {
    a + b == b + a
}

prove fn noOverflow(a: i32, b: i32) -> bool 
    requires a >= 0 && b >= 0
    requires a + b < i32.MAX
{
    let sum = a + b;
    sum >= a && sum >= b
}

// Integration with proof assistants
extern proof "Coq" "./theories/GroupTheory.v"
extern proof "Lean" "./theories/Monoid.v"
```

### D.13 Type-Driven Serialization

Zero-boilerplate serialization:

```vantor
#[serializable(format: "json", "msgpack", "protobuf")]
struct User {
    id: i64,
    name: String,
    email: Email,  // Validated at deserialization
    createdAt: DateTime,
    roles: Vec<Role>,
}

let user = User::fromJson(jsonString)?;
// or
let bytes = user.toMsgPack()?;
let proto = user.toProtobuf()?;
```

### D.14 REPL with Hot Compilation

Interactive development with instant feedback:

```bash
$ vantor repl
VantorLang 1.0.0 - Interactive Mode

> let x = 42
x: i32 = 42

> fn double(n: i32) = n * 2
defined: fn double(n: i32) -> i32

> double(x)
res: i32 = 84

> :load math.vnt
Loaded math.vnt

> :time
Timing: 0.001ms

> :mem
Memory: 2MB
```

### D.15 Incremental Compilation

Extremely fast incremental builds:

```vantor
// Vantor tracks dependencies at function level
// Rebuilds only what's changed

// First build: 10s
// After changing one function: 0.1s

// Cached compilation artifacts
cache/
  ├── target/
  │   └── debug/
  │       ├── deps/
  │       ├── examples/
  │       └── incremental/
  └── .vantor/
      └── cache.db  // Function-level dependency graph
```

### D.16 Built-in Package Manager with Security

```bash
# Audit dependencies for vulnerabilities
vantor audit

# Automatically update to secure versions
vantor update --security

# Sign your packages
vantor sign --private-key key.pem

# Reproducible builds
vantor build --reproducible
```

---

## Appendix E: Performance Targets

| Benchmark | VantorLang | Rust | Go | Java | C# |
|-----------|-------------|------|-----|------|-----|
| Fibonacci (1M) | 0.8x | 1.0x | 2.5x | 3.0x | 2.8x |
| JSON Parse | 0.9x | 1.0x | 1.8x | 2.2x | 1.9x |
| HTTP Server (req/s) | 1.1x | 1.0x | 0.9x | 0.7x | 0.8x |
| Cold Start | 0.1x | 0.1x | 0.5x | 2.0x | 1.5x |
| Memory Usage | 0.6x | 0.5x | 1.0x | 2.0x | 1.8x |

---

## Appendix F: Roadmap Detalhado

### Fase 1: MVP (Meses 1-6)
- [x] Lexer
- [x] Parser básico
- [x] CLI
- [ ] Parser completo
- [ ] Type checker básico
- [ ] Executor interpreter

### Fase 2: Type System (Meses 7-12)
- [ ] Genéricos completos
- [ ] Traits/Interfaces
- [ ] Null safety
- [ ] Pattern matching
- [ ] Type inference

### Fase 3: Concorrência (Meses 13-18)
- [ ] Async/Await
- [ ] Task runtime
- [ ] Channels
- [ ] Actors

### Fase 4: Otimizações (Meses 19-24)
- [ ] LLVM backend
- [ ] Inlining
- [ ] LTO
- [ ] SIMD

### Fase 5: Ecossistema (Meses 25-30)
- [ ] VPM (Package Manager)
- [ ] Stdlib completa
- [ ] Drivers (DB, HTTP)
- [ ] Testing framework

### Fase 6: IDE & Tools (Meses 31-36)
- [ ] LSP Server
- [ ] VSCode plugin
- [ ] Debugger
- [ ] Formatter
- [ ] Release 1.0

---

*Document Version: 1.1*
*Advanced Features Added: 2026-02-28*

