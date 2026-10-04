# Troubleshooting - Lexicon v0.3.5

## 1. Common issues (real)

### 'lex' command not recognized

- **Cause**: `~/.lexicon/bin` is not on this session's PATH yet.
- **Fix**: reopen the terminal and run `lex install` (idempotent — never
  duplicates). On Windows the PATH persists in the user registry and needs
  a fresh session.

### `file.lex:line:col: msg` before anything runs

- **Cause**: the pre-run gate (or `lex check`/`lex vet`) found a syntax/type error.
- **Fix**: not a runner bug — fix the indicated `line:col`. Usual suspects:
  E0101 (unclosed string), E0201 (expected token, e.g. `;`), E0301 (type
  mismatch). Tip: `lex fix --dry-run` previews safe migrations
  (e.g. `Http.get(` → `Http::get(`).

### Hot reload stopped: "child keeps dying fast"

- **Cause**: the child died in <1 s three times (busy port is the classic).
- **Fix**: free the literal `Http::serve("0.0.0.0:3000")` port or fix the
  error, then restart `lex run --watch`. Backoff is protection, not a bug.

### `Env::get` returns `NOT_FOUND`

- **Cause**: process started without env loaded — correct behavior, not an error.
- **Fix**: run via `demo-api\run-dev.ps1` / `run-prod.ps1`; and assign the
  variable first (`let x = Env::get(..)`), since inline-in-`print` renders literally.

### Commented-out code "does nothing" / route disappeared

- **Cause**: none — that is correct in v0.3.5. Comments are stripped before
  runner/lint/scan; commented `print` or `Http::serve` never runs nor registers.

### No IntelliSense/vet in VS Code

- **Cause**: old extension or `lex` off PATH (`lexicon.path`).
- **Fix**: update **Lexicon Super** to 1.2.0, run `lex install`, and check
  `lex vet <file>` in the terminal — vet-on-save mirrors that result.

### `lex deploy` "succeeded" but nothing published / `lex ffi` with no bindings

- **Cause**: both are **simulated** in v0.3.5 (cosmetic output).
- **Fix**: do not use them in production; follow the roadmap for real status.

## 2. Glossary

- **AST**: syntax tree built by the lexer/parser from `.lex`.
- **Backoff (hot reload)**: stop after 3 fast child deaths (<1 s).
- **Comment-accurate runner**: runner operates on comment-free code.
- **Debounce (300 ms)**: coalescing saves into one restart.
- **E-codes (E0101–E0801)**: stable diagnostic codes for CI/IDEs.
- **FFI**: C/Rust interop (`lex ffi` currently simulated).
- **Hot Reload**: `lex run --watch` — save → supervised auto-restart.
- **LEX_SUPERVISED=1**: supervised-child environment marker.
- **Pipe (`|>`)**: functional data-flow chaining.
- **Pre-run gate**: extension syntax barrier before executing.
- **Vet (`lex vet`)**: static correctness checks (type/interface/ABI).
- **Vet-on-save**: vet diagnostics inside the editor on save.
- **WASM**: WebAssembly target (`--target wasm` in scaffolds).
