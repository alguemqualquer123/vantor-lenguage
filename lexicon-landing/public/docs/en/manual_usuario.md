# User Manual - Lexicon v0.2.0

> Lexicon **v0.2.0**: hot-reload supervisor, VS Code super extension **1.1.x**,
> comment-accurate runner, and working `lex check` / `lex vet`.

## 1. Introduction

Lexicon is a modern language with a real Rust toolchain: `lex run`,
`lex check`, `lex vet`, `lex test`, `lex lint`, `lex fmt`, `lex doc` and
`lex trace`. This manual covers the portal and the daily workflow.

## 2. Portal tour (landing page)

- **Hero**: language pitch, **v0.2.0** badge, download + repository buttons.
- **Features**: hot-reload supervisor, super extension, pre-run diagnostics,
  honest installer.
- **Templates + Quickstart**: `lex new ...` scaffolds plus copyable real
  commands (`lex run --watch`, `lex vet`, demo-api flow).
- **Downloads**: Windows (`.msi`), Linux (`.tar.gz`), macOS (`.pkg`), WASM
  (`.wasm`) — all **v0.2.0**.
- **Docs**: this manual, installation, architecture, troubleshooting.

## 3. Using the CLI (lex)

```powershell
lex new my-api --template api --edge   # project scaffold
lex run main.lex                       # compile and run
lex run --watch main.lex               # hot reload: save → restart
lex check main.lex                     # type checking, no build
lex vet models.lex                     # correctness gate (expect: "vet: no correctness violations")
lex test                               # @Test tests
lex lint --json                        # machine-readable lint
lex fmt                                # formatting
lex doc routes.lex                     # API docs as Markdown
lex trace main.lex                     # stage timings
lex install                            # global install (idempotent)
```

### Real syntax (`::` form)

```lex
import core::net::Http;
import core::json::Json;
import app::models;
import app::store;

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
```

> Imports use `::` (`core::net::Http`). The dotted form is fixed by `lex fix`,
> but write `::` directly.

## 4. Hot Reload (`lex run --watch`)

The supervisor **never runs user code in-process**: it spawns a child
`lex run <file>` (no `--watch`, no recursion) and restarts it on every save.

- **Debounce**: save bursts coalesce (300 ms) into **one** restart.
- **Filter**: only `.lex` triggers; `target/`, `build/`, `.git/`, dotfiles,
  backups (`~`), `.tmp`/`.swp`, `*.db*` are ignored.
- **Backoff**: if the child dies in <1 s three times (e.g. busy port), the
  supervisor stops instead of hot-spinning.
- **Env inheritance**: the child inherits `APP_ENV`, `DATABASE_URL`, etc. —
  dev/prod keeps working under `--watch` (`LEX_SUPERVISED=1` in the child).
- **CI**: with `CI=true`, watch becomes a single supervised run.

## 5. VS Code Super Extension 1.1.x

- **85+ snippets** that are Lexicon-accurate (`main`, `fn`, `@Get`, `serve`,
  `match`, `struct`, `@Test`, `Http::get`, `Json::parse`, `print`, `/* */` block).
- **4 themes**: Dark Pro, Midnight, Light, High Contrast (+ icon theme).
- **Contextual autocomplete + auto-import**: `Module::` lists only that
  module's members; accepting a suggestion inserts the missing
  `import core::x::Y;`.
- **Pre-run syntax gate**: E0101/E0201 check with exact `line:col` before
  running (*Show Problems* / *Run Anyway*); same findings in save diagnostics.
- **Vet-on-save**: `lex vet` diagnostics in-editor + `lexicon-vet` problem
  matcher for tasks (`lex run --ci`, `lex run --watch`, `lex test`, `lex check`).
- No extension? `lex ide init` scaffolds `.vscode/tasks.json`, snippets, `LEX-TOOLS.md`.

## 6. Real demo API (HTTP + SQLite + .env)

```powershell
powershell -ExecutionPolicy Bypass -File demo-api\run-dev.ps1    # env=development, sqlite:./dev.db
powershell -ExecutionPolicy Bypass -File demo-api\run-prod.ps1   # env=production,  sqlite:./prod.db
curl http://localhost:3000/users
curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"Bob","email":"bob@example.com"}'
```

- `python3 demo-api/seed.py demo-api/dev.db` creates the real DB (4 users).
- `Env::get("NAME")` reads the **real** environment: assign first
  (`let x = Env::get(..)`); inline inside `print` renders literally.
- `Http::serve("0.0.0.0:3000")` binds the **literal** address; `.env`
  switches behavior (env/db/log), not the port.

## 7. Honest notes (known limits)

- `lex run` executes **one file**; sibling modules demonstrate imports/exports
  and are validated by `vet`/`lint`/`fmt`/`doc`.
- Comments (`//`, `/* */`) are truly ignored: commented code never runs,
  never registers routes, never starts servers.
- `lex deploy` and `lex ffi` are currently **simulated** (progress bars, no
  real effect) — do not treat them as production deploy/bindings.
- Native WebView support was removed from the binary (no MinGW-GNU link);
  nothing in the `check`/`build` flow depends on webview.
