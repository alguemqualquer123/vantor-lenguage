# Missing Implementations – Super Lingua

## Overview
This document summarizes the features and components that **have not yet been implemented** in the Super Lingua project. Items marked with `[ ]` in the original `PLANO_SUPER_LINGUAGEM.md` are still pending.

> Verified against code in v0.2.0 (language; VS Code extension is 1.1.x,
> versioned separately): boxes flipped to `[x]` below were proven in
> `src/lexicon-cli/src/compiler.rs`, `src/lexicon-lexer/src/comments.rs`,
> `src/lexicon-cli/src/installer.rs`, `demo-api/`, or
> `themes/lexicon-vscode/` (README + CHANGELOG). Everything else stays
> missing and is summarized in [Roadmap pós-0.2.0](#roadmap-pós-020).

## Language Specification (Phase 1)
- [ ] Finalize minimal language spec (syntax, modules, imports).
- [ ] Define module/package system and import semantics.
- [ ] Complete grammar/EBNF for the full language.
- [ ] Add support for additional literals (e.g., multi-line strings, raw strings).

## Lexer (Phase 3)
- [ ] Implement token types for all operators and punctuation.
- [ ] Handle escape sequences in strings.
- [ ] Add support for Unicode identifiers if required.
- [x] Comment handling (`//`, `/* */`) — `strip_comments`/`code_contains` with unit tests; runner, lint and security scan comment-free code.
- [ ] Provide comprehensive tokenization tests and fuzzing (partial: in-tree fuzz corpus wired into `lex test`/`lex bench` via `fuzz_target`; full harness pending).

## Parser & AST (Phase 4)
- [ ] Implement full set of AST node types (including modules, imports, generics).
- [ ] Choose and fully implement a parsing strategy (e.g., Pratt parser) for expressions.
- [ ] Add parsing for all statements (if, while, for, switch, match, etc.).
- [ ] Implement full error recovery and user‑friendly diagnostics.
- [ ] Generate complete AST for all valid programs.

## Semantic Analysis (Phase 5)
- [ ] Complete symbol table with nested scopes (global, function, block).
- [ ] Implement generic type system (generics, traits/interfaces).
- [ ] Add full type checking for expressions, functions, and method calls.
- [ ] Enforce rules for implicit/explicit casts.
- [ ] Detect unused declarations and emit optional warnings.
- [ ] Implement runtime type information for error messages.

## IR Generation & LLVM Backend (Phase 6)
- [ ] Map all language types to LLVM IR types (structs, arrays, vectors).
- [ ] Emit IR for complex constructs: closures, async/await, pattern matching.
- [ ] Integrate LLVM passes for `-O0`, `-O1`, `-O2`, `-O3` with performance profiling.
- [ ] Generate debug info (`-g`) and enable IR dumping (`--emit-llvm`).

## Runtime & `Console::` (Phase 7)
- [ ] Implement full `Console` API (`log`, `error`, `write`, `writeln`, `readLine`).
- [ ] Provide C‑ ABI compliant bindings for all runtime functions.
- [ ] Add conversion utilities between language strings and C strings.
- [ ] Add additional runtime modules (files, time, environment) as planned.

## Logging System (Phase 8)
- [ ] Complete logger configuration (`--log-level`, `--log-file`).
- [ ] Implement structured logging for compiler phases and runtime calls.
- [ ] Add log rotation and file rollover support.
- [ ] Benchmark logging overhead and expose a “quiet” mode.

## Performance & “Perfect‑Fast Compilation” (Phase 9)
- [ ] Profile each compilation phase and expose timing flags.
- [ ] Optimize lexer buffer handling and minimize dynamic allocations.
- [ ] Reduce parser backtracking and simplify grammar where possible.
- [ ] Tune LLVM pass selection for fastest code generation vs. compile time trade‑off.

## Developer Experience (Phase 10)
- [x] Full CLI (`lex`, clap-based) with help (`--help`) and `version` command plus flags (`--watch`, `--ci`, `--features`, `--target`, …).
- [ ] Add friendly error messages with code snippets and caret pointers.
- [x] IDE integration: VS Code super extension 1.1.x (TextMate grammar, contextual completions + auto-import, hover, diagnostics, pre-run gate). Full LSP server (`lex lsp` entry exists) still pending.
- [ ] Set up CI pipeline with unit, integration, and fuzz tests.
- [ ] Write comprehensive user documentation, examples, and tutorials.

## Tests & CI
- [ ] Expand unit‑test suite to cover all language constructs.
- [ ] Add integration tests that compile and run end‑to‑end examples.
- [ ] Configure automated CI (build, test, lint, coverage).

## Miscellaneous
- [x] Create and maintain a `Docs/` directory with all project documentation (exists; refreshed to v0.2.0 implemented reality).
- [ ] Migrate all existing Markdown files into the new `Docs/` folder.
- [ ] Remove stale documentation files from original locations.

## Roadmap pós-0.2.0

Verified-done, do not reopen: `--watch` supervisor, comment-accurate
runner/lint/security, silent idempotent installer with registry dedup,
webview dep removal (MinGW link), extension 1.1.x feature set,
`demo-api/` (real axum + SQLite via `lexicon-db` + dev/prod env),
working `check`/`vet`, `lex mod` package manager with `lexicon.lock`
(0.3.2), native graphics engine on wgpu — `Window`/`Canvas`/`Input`/`Gpu`
with Vulkan/DX12/OpenGL/Metal backends (0.3.2), one-binary SDK with
~40-byte launcher shims (0.3.4), per-package symbol resolution in the
interpreter (0.3.4).

Genuinely missing: real cloud deploy, real FFI bindings, WASM
auto-bindings, procedural-macro sandbox, full debugger, real registry
upload, LLVM backend beyond IR-text stub, GC/scheduler runtime,
full LSP server, playground,
supply-chain tooling, advanced type-system/concurrency/FP features.

## Stdlib Go-parity — estado em 0.3.4

`lib/std` tem **89 pacotes** escritos em Lex, todos embarcados no SDK
(`lex sdk export`) e checados por `lex check`. Quarta onda (0.3.4):
`crypto/rc4`, `math/cmplx`, `net/netip` novos; `crypto/md5`, `crypto/sha1`,
`mime/quotedprintable`, `sync/pool`, `text/tabwriter`, `encoding/binary`,
`net/http` e `native/{window,canvas,input,gpu}` passaram a ser exportados.

O que **ainda não** existe, e por quê:

| Pacote Go | Bloqueio |
|---|---|
| `net`, `net/rpc`, `database/sql`, `crypto/tls` | exigem sockets/TLS reais no nativo; `net/http` hoje é o cliente embutido do motor |
| `sync` (WaitGroup/Mutex/Once concorrentes) | não há scheduler de threads no interpretador; o que existe é semântica single-thread |
| `context` com cancelamento assíncrono | idem — sem goroutines |
| `reflect` completo (Field/SetValue, TypeByID) | só `typeOf`/`typeof`/`type` (0.3.1) |
| `os` com processos/forks reais | `os/exec` chama o `Command` nativo; `os/user` e `os/signal` são aproximados |
| `image`, `image/png`, `go/ast`, `go/parser` | fora do plano de paridade; `image/color` cobre paletas |
| `crypto/{rsa,ecdsa,ed25519,aes,rand}` | dependem do gate `sign`/`tls` do `Cargo.toml` (FEATURE_MATRIX) |

Regra de escrita de pacote (vale para qualquer onda futura): o namespace de
imports é **achatado por nome simples**, então chamadas dentro de um pacote
devem ficar qualificadas entre pacotes (`strings::Index`, não `Index`); o
próprio pacote sempre vence o próprio nome desde 0.3.4. Operadores `|` e `~`
não existem no parser — use `bitor(a,b) = a + b - (a & b)` e
`NOT x = 4294967295 - x`. Divisão float por zero é erro de runtime, não
`NaN`. Listas são **valor**: um helper não consegue mutar a lista do
chamador.
