# Architecture and APIs - Lexicon v0.3.5

## 1. System architecture

Crates (Rust workspace): `lexicon-cli`, `lexicon-lexer`, `lexicon-parser`,
`lexicon-analysis`, `lexicon-codegen`, `lexicon-core`, `lexicon-utils`,
`lexicon-async`, `lexicon-http`, `lexicon-cache`, `lexicon-db`, `lexicon-wasm`,
`lexicon-gui` (behind the `gui` feature).

### 1.1 Compilation pipeline

1. **Lexer** (`lexicon-lexer`): tokenizes `.lex`, including `::` (PathSep),
   `{var}` interpolation, `@Get/@Post/@Put/@Delete/@Test` decorators, `#[cfg]`.
2. **Parser** (`lexicon-parser`): AST + **visible-error recovery** — errors
   surface as `file:line:col: msg` and **fail** (no silent fallback).
3. **Analysis** (`lexicon-analysis`): type checker, `#[cfg]` by features/target,
   borrow rules (E0305/E0382).
4. **Codegen** (`lexicon-codegen`): LLVM IR backend (`build/output.ll`).

`lex check <file>` runs this pipeline without emitting a build; `lex build`
accepts `--release`, `--target`, `--features`.

### 1.2 Hot reload supervisor (`lex run --watch`, Spec §62)

- The supervisor never runs user code: it spawns a child `lex run <file>`
  with `LEX_SUPERVISED=1` and kills/waits (`kill`+`wait`, kill-on-drop) per restart.
- 300 ms debounce, `.lex` filter (ignores `target/`, `build/`, `.git/`,
  dotfiles, `~`, `.tmp`/`.swp`, `*.db*`), 3×<1 s death backoff.
- Inherits the environment (dev/prod `.env` keeps working under `--watch`).

### 1.3 Comment-accurate runner

`strip_comments`/`code_contains` mirror the tokenizer: commented `print`,
`Http::serve` and routes **never** execute, register, or start servers.
Runner, lint and security scan operate on comment-free code.

## 2. Diagnostics: `lex check`, `lex vet`, E-codes

`lex vet [file]` = type check + interface + ABI; clean prints
`vet: no correctness violations` (security findings are advisory).
Every diagnostic carries a stable code (`E0101`, `E0201`, …) for CI/IDEs.

| Code | Meaning |
|---|---|
| E0101 / E0102 / E0103 | unclosed string / invalid literal / unexpected char |
| E0201 / E0202 / E0203 | expected token / invalid assignment / unreachable code |
| E0301 / E0302 / E0303 / E0304 | type mismatch / unknown type / generic constraint / unimplemented interface |
| E0305 / E0382 | borrow violation / use-after-move |
| E0306 | narrowing requires `as` cast |
| E0401 | codegen failure |
| E0501 | runtime panic |
| E0601 | IO/fs/compression error |
| E0701 | concurrency race (closed channel, backpressure, data race) |
| E0801 | unwrap on `None` |

The extension adds the pre-run gate (E0101/E0201 with `line:col`) plus
W0001/W0002 warnings, with the same findings in save diagnostics.

## 3. Runtime APIs (real)

- `Http::serve("0.0.0.0:3000")`: starts the real HTTP server (axum, CORS,
  JSON envelopes, 404/405, request log) — blocks until killed, ideal for `--watch`.
  The address is a **literal**: that is what gets bound.
- `Http::get/post`, `Json::parse`, `Env::get`, `Db::connect/query/execute`,
  `Console::writeLine`, `print/println/panic/recover/assert/inspect/spawn`.
- `lex complete --prefix "Http::"` / `--file … --line … --col … [--json]` and
  `lex lsp` (LSP 3.17 over stdio: initialize/completion/hover) share one
  registry — what autocomplete suggests, the toolchain honors.

## 4. Demo API (demo-api/)

`main.lex` (runnable via `lex run`), `models.lex` (`pub struct`/`pub type`/
`pub fn`), `config.lex` (`Env::get`), `store.lex` (`Db::connect/query/execute`
via `lexicon-db`/SQLite), `routes.lex` (`@Get/@Post` handlers), `.env.dev` /
`.env.prod`, `run-dev.ps1` / `run-prod.ps1`, `seed.py` + real `dev.db`.

## 5. One binary, one SDK

The published `lex` is **one** executable (~9.4 MB in the `dist` profile,
`opt-level="z"` + fat LTO). The 38 launchers (`lex-run`, `lex-check`, `lex-mod`,
…) are ~40-byte shims that forward the subcommand to their neighbour — they are
not 38 copies of the binary.

- The stdlib, templates and examples are **embedded** in the binary
  (`include_str!`): `lex sdk export` writes the kit to `build/sdk` (or wherever
  you point it) and the auto-install replicates it into `~/.lexicon/sdk`.
- `lex sdk verify` checks the 116 required files and the size budget
  (mini ≤ 6 MB, full ≤ 16 MB) — a new dependency that blows the budget fails it.
- `lex sdk info` prints flavour, path and size of the binary in use.

### 5.1 GUI engine (`lexicon-gui`, `gui` feature)

`Window` / `Canvas` / `Input` / `Gpu` are interpreter natives
(`src/lexicon-cli/src/gfx.rs`) on top of **eframe + wgpu**: the backend is
chosen by your GPU (Vulkan, DX12, Metal, OpenGL, WebGPU). `Canvas::clear/
fillRect/fillCircle/line/text`, `Input::keyDown(win, "left")` (canonical key
names) and `Window::backend(win)`. Under `lex run --ci` the window closes
itself after ~1 s (`LEXICON_GUI_AUTOQUIT=<frames>`).

## 6. Honest v0.3.5 limits

- `lex deploy` and `lex ffi` are **simulated** (fake progress, no real effect).
- `lex run` executes **one file**; the module graph is validated, not linked.
- No native WebView in the binary (`webview` crate removed — no MinGW-GNU
  link); `compiler.rs`/`webview.rs` remain unlinked stubs.
- WASM AOT/JIT and a public package registry remain on the roadmap; `lex mod`
  already resolves `github:`/`gitlab:`/`https:`/`path:` and writes `lexicon.lock`.
- `lex build` still emits only an LLVM IR stub (`define i32 @main() { ret i32 0 }`)
  and does **not** produce a native executable: `lex run` is a tree-walking
  interpreter, and whatever it cannot evaluate fails as a runtime error — only
  `lex check`/`lex vet` act as the pre-run gate.
