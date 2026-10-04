# Super Language Design Document

## Vision
Create a modern, statically‑typed compiled language that matches or exceeds Go‑level performance, developer ergonomics, and ecosystem maturity.

## Core Language Features
- **Syntax & Types**: clear, consistent syntax; static typing with inference; structs, enums, unions, tuples, interfaces, generics (with constraints), pattern matching (optional).
- **Memory Model**: default concurrent garbage collector with optional manual allocation, ownership/borrowing model, smart pointers.
- **Concurrency**: lightweight goroutine‑like tasks, async/await, channels (buffered/unbuffered), scheduler, atomic primitives, mutexes/RWMutexes, semaphores, work‑stealing, structured concurrency, cancellation, race detector.
- **Error Handling**: error interfaces, `Result<T,E>` / `Option<T>` types, error wrapping, stack traces, panic/recover.
- **Modules & Packages**: hierarchical module system, import/export, versioning (semver), lock file, dependency graph, vendor mode, private/internal packages.
- **Reflection & Metaprogramming**: runtime type information, compile‑time execution, macros/procedural macros, attributes, code generation.

## Runtime
- Scheduler, task pool, GC (generational, concurrent, low‑pause), C‑ABI bindings, FFI for C/C++/Rust/JS, safe `unsafe` block, SIMD intrinsics, inline asm.
- Built‑in support for OS abstractions: processes, threads, signals, environment, file system, networking, timers.

## Standard Library (Bootstrap)
- **os / fs**: files, directories, permissions, watches.
- **io**: streams, buffers, readers/writers.
- **net**: TCP/UDP, Unix sockets, HTTP/2/3, WebSocket, TLS, gRPC, DNS.
- **crypto**: modern primitives (AES‑GCM, ChaCha20‑Poly1305, RSA, ECC, Ed25519, X25519, hash functions, HMAC).
- **encoding**: JSON, XML, YAML, TOML, protobuf, MessagePack, CBOR, base64, hex.
- **sync**: mutexes, RWMutexes, atomic, condition variables, wait groups.
- **time**: timestamps, durations, time zones, monotonic clocks.
- **fmt**: formatted I/O, string interpolation.
- **testing**: unit, integration, fuzz, property‑based, benchmarks, mock framework.
- **database**: generic SQL driver, connection pool, ORM‑lite, migrations.
- **web**: HTTP server/client, routing, middleware, sessions, cookies, websockets.
- **security**: secure random, password hashing, constant‑time ops, sandboxing utilities.

## Toolchain
- **Compiler**: fast incremental front‑end, LLVM‑based backend (x86‑64, ARM64, RISC‑V, WebAssembly), LTO, PGO, debug symbols.
- **Build System**: parallel compilation, caching, cross‑compilation, reproducible builds, CI integration.
- **Package Manager**: `lang add/remove/update/publish`, lock file, checksum verification, semantic versioning, registry.
- **Formatter/Linter**: deterministic `gofmt`‑style formatter, lint rules for performance, security, style.
- **Debuggers & Profilers**: LLDB/GDB integration, remote debugging, CPU/memory/GC profiling, flame graphs.
- **IDE Support**: VS Code extension, Language Server Protocol (completion, hovers, diagnostics, formatting, go‑to‑definition, rename).
- **Playground**: web REPL with sandboxed execution, shareable URLs.

## Security & Supply‑Chain
- Signed releases, SBOM generation, reproducible builds, dependency provenance, vulnerability scanning, security advisories.

## Performance Targets
- Compile time < 1 s for typical project, incremental builds milliseconds.
- Runtime overhead < 5 % vs C for comparable workloads.
- GC pause ≤ 1 ms, low‑latency scheduler, zero‑cost abstractions.
- Benchmarks suite covering compilation, runtime, GC, I/O, networking.

## Milestones
1. **Language Spec** – full grammar, type system, error model.
2. **Lexer/Parser/AST** – tokenization, parsing, AST validation.
3. **Type Checker & Generics** – inference, constraints, monomorphization.
4. **IR & LLVM Backend** – SSA IR, optimization passes, codegen for major targets.
5. **Runtime & GC** – scheduler, GC, task system, C‑ABI.
6. **Standard Library Bootstrap** – core modules listed above.
7. **Package Manager & Build** – CLI, lock file, reproducible builds.
8. **Toolchain Extras** – formatter, linter, test runner, debugger, profiler.
9. **Documentation Site** – language spec, tutorials, API docs, examples.
10. **IDE/LSP** – VS Code extension, language server.
11. **Playground & Examples** – web REPL, sample projects.
12. **Security & Compliance** – supply‑chain, audits.
13. **Performance Benchmarking** – automated benchmark pipeline.

## Short‑Term Tasks
- Draft complete language specification (syntax, types, modules, error handling).
- Implement lexer, parser, AST builder.
- Build type checker with generic support.
- Create initial IR skeleton and integrate LLVM backend.
- Develop minimal runtime with GC and scheduler.
- Bootstrap standard library (io, fmt, net, crypto, time, sync).
- Implement package manager CLI (`lang add`, `remove`, `publish`).
- Provide official formatter and linter.
- Set up CI/CD (GitHub Actions) for build, test, lint, benchmark.
- Publish initial documentation site.
- Prototype VS Code extension (syntax highlight, IntelliSense).

## Long‑Term Goals
- Full ecosystem: public package registry, playground, CI templates.
- Governance model: RFC process, versioning policy, deprecation roadmap.
- Stable ABI across releases, binary compatibility.
- Expand targets: embedded, mobile, WASM, GPU (CUDA/OpenCL).
- Advanced language features: async/await, structured concurrency, advanced macros, reflection, ownership/borrowing, RAII, SIMD intrinsics.
- Security tooling: static analysis, fuzzing, supply‑chain scanning.
- Performance engineering: profile‑guided optimization, tiered compilation, zero‑cost abstractions.
- Comprehensive testing: cross‑platform CI, fuzzing harness, regression suite.

## Architecture Overview
```
Language Layer
 └─ Lexer → Parser → AST → Type Checker → IR
Backend Layer
 └─ LLVM (or native) → Optimizer → Codegen → Linker
Runtime Layer
 └─ Scheduler → GC → Task System → FFI
Standard Library
 └─ OS, IO, Net, Crypto, Encoding, Sync, Time, Testing, DB
Toolchain
 └─ Compiler, Build System, Package Manager, Formatter, Linter, Debugger, Profiler
Ecosystem
 └─ VS Code Extension, LSP, Playground, Registry, Docs Site
```

*All sections above should be expanded with concrete design decisions, data structures, algorithms, and API signatures as the project progresses.*

## Implemented reality in v0.2.0 (verified in code)

> Language is **v0.2.0** (workspace `Cargo.toml`); the VS Code extension
> is versioned separately (**1.1.x**).

- **Hot-reload supervisor** (`lex run --watch`, `compiler.rs::watch`):
  child `lex run` process + 300 ms debounce + `.lex`-only filter
  (`target/`, `build/`, `.git/`, dotfiles, `*.db*` ignored) + backoff
  after 3 deaths under 1 s.
- **Comment-accurate tooling**: `strip_comments`/`code_contains`
  (`lexicon-lexer/src/comments.rs`, unit-tested); `run`, `lint` and the
  security scan operate on comment-free code with stable line numbers.
- **Silent idempotent installer** (`installer.rs::auto_install`, run at
  every `lex` startup, skipped in CI): copies the binary only when
  changed and appends to PATH only when absent (process env + Windows
  registry check with re-check before writing, so no duplicates).
- **Webview dep removed (2026-09-29)**: unlinked dead code + stubbed
  runner paths; enables MinGW linking (`ole32` symbols were missing).
- **VS Code super extension 1.1.x**: grammar, 85+ snippets, themes,
  toolchain commands, contextual autocomplete + auto-import, hover docs,
  go-to-definition/symbols, vet-on-save diagnostics, pre-run syntax gate.
- **demo-api/**: real axum HTTP server (CORS, JSON envelopes, 404/405)
  with decorator routes, SQLite via `lexicon-db` (sqlx), dev/prod `.env`
  via `run-dev.ps1`/`run-prod.ps1`, working `lex check`/`vet`.

## Roadmap pós-0.2.0

Genuinely missing (current commands are simulated stubs or partial):
real Lexicon Cloud deploy, real C/Rust/Python FFI bindings, WASM with
automatic JS bindings, full procedural-macro sandbox, full
breakpoints/DWARF/PDB debugger, registry with real `publish` upload,
plus the milestones above not yet implemented (LLVM backend beyond the
IR-text stub, GC/scheduler runtime, package manager with lockfile,
LSP server beyond the extension side, playground, supply-chain tooling,
benchmark pipeline).