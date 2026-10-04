# Roadmap: JS/Node.js Parity Implementation

This document tracks the implementation of ECMAScript and Node.js parity features within the Lexicon language. The goal is to provide the utility and API surface of JS/Node while maintaining Lexicon's performance and static typing.

## Phase 1: Syntax & Semantics Expansion
- [ ] **Operators**:
    - [ ] Optional Chaining (`?.`)
    - [ ] Spread/Rest (`...`)
- [ ] **Async & Generators**:
    - [ ] `async` / `await` keywords
    - [ ] `yield` keyword & Generator functions
- [ ] **Control Flow**:
    - [ ] `for...of` (Iterator protocol)
    - [ ] `for...in` (Object property iteration)
    - [ ] `try...catch...finally` (Structured exception handling)

## Phase 2: Core Objects & fundamental Types
- [ ] **Dynamic Objects**:
    - [ ] Implement a `DynamicObject` type (HashMap-backed)
    - [ ] Implement `Object` static methods (`keys`, `values`, `entries`, `assign`, etc.)
- [ ] **Collections**:
    - [ ] `Array` method suite (map, filter, reduce, slice, splice, etc.)
    - [ ] `Map` and `Set` primitives
    - [ ] `WeakMap` and `WeakSet`
- [ ] **Fundamental APIs**:
    - [ ] `Math` library
    - [ ] `Date` and `Time` (including Temporal proposals)
    - [ ] `String` and `Number` utility methods
    - [ ] `RegExp` (Regular Expressions)

## Phase 3: Async Runtime & Concurrency
- [ ] **Async Model**:
    - [ ] `Promise` abstraction (wrapping native tasks)
    - [ ] Event Loop & Microtask Queue
- [ ] **Iterators**:
    - [ ] `Symbol.iterator` and `Symbol.asyncIterator` protocols
    - [ ] Generic iteration support

## Phase 4: System Standard Library (Node.js Parity)
- [ ] **File System (`fs`)**:
    - [ ] Async/Sync I/O
    - [ ] Directory management
    - [ ] File watching
- [ ] **System Modules**:
    - [ ] `os` (system info)
    - [ ] `path` (cross-platform manipulation)
    - [ ] `util` (general utilities)
- [ ] **Events & Streams**:
    - [ ] `EventEmitter` (Event-driven architecture)
    - [ ] `stream` (Readable, Writable, Duplex, Transform)
- [ ] **Process & Environment**:
    - [ ] `process` global (env, argv, signals, pid)
    - [ ] `buffer` (Binary data handling)

## Phase 5: Advanced Runtime & Interop
- [ ] **Reflection & Metaprogramming**:
    - [ ] `Proxy` (intercepting operations)
    - [ ] `Reflect` API
- [ ] **Parallelism**:
    - [ ] `worker_threads` (OS threads)
    - [ ] `cluster` mode
- [ ] **Isolation**:
    - [ ] `vm` module (isolated execution contexts)
