# Lexicon Super Extension (v1.0.0)

Super extension for the **Lexicon** programming language (`.lex`, `.lang`):
full TextMate grammar, **85 snippets**, 3 color themes + icon theme,
13 `lex` toolchain commands, hover/completion/go-to-definition/symbols,
Run/Test CodeLens, vet-on-save diagnostics and task definitions.

## Install

```bash
# from this folder (requires node + @vscode/vsce):
npx @vscode/vsce package --no-dependencies
code --install-extension lexicon-super-1.0.0.vsix
```

Requires the `lex` binary on `PATH` (or set `lexicon.path`).
Build it from the repo root — note: linking needs MSVC on Windows
(MinGW lacks `ole32` symbols for the `webview` dep); `cargo check`
validates everything without linking.

## Features

| Area | What you get |
|---|---|
| Grammar | `source.lexicon`: all keywords from `tokens.rs`, `int/uint/byte/rune/decimal`, hex/binary/float literals, `{var}` interpolation, `///` docs, `@Get/@Post/@Put/@Delete/@Test`, `#[cfg(...)]`, `\|>` pipes, closures, generics, paths |
| Snippets | 85 prefixes — see [SNIPPETS.md](./SNIPPETS.md) (generated from the JSON) |
| Themes | Lexicon Dark Pro, Lexicon Midnight, **Lexicon Light** (new) + Lexicon Icons |
| Commands | Run, Run with Args, Check, Vet, Lint, Format, Doc, Trace, Test, Build, Debug, New Project, Show Snippets, Env |
| IntelliSense | keyword/type/stdlib (`Http`, `Json`, `Env`, `Db`, `List`, `Map`, `Option`, `Result`, `Channel`) completions, hover docs, go-to-definition, document symbols |
| Diagnostics | `lex vet` on save → Problems panel (respects `lexicon.vetOnSave`) |
| Tasks | `"type": "lex"` task definitions (`run/build/test/check/vet/lint/bench`) |

## Snippet highlights (prefix → expansion)

- `api` — minimal runnable HTTP API (`import`, `Env::get`, `@Get`, `Http::serve`)
- `get/post/put/del` — route handlers; `serve` — server line
- `dbcon/dbq/dbx/dbi` — SQLite via `Db::connect/query/execute`
- `printenv` — the `let x = Env::get(..)` pattern that really evaluates
- `switch`, `matchg` (guard), `mbool` (exhaustive), `defer`, `vfn` (variadic)
- `test` — `@Test` function for `lex test`

Full catalog: [SNIPPETS.md](./SNIPPETS.md).

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `lexicon.path` | `lex` | `lex` binary location |
| `lexicon.features` | `""` | `--features` for `build/check/vet` (`#[cfg(feature)]`) |
| `lexicon.target` | `""` | `--target` override for `build` |
| `lexicon.vetOnSave` | `true` | Diagnostics on save |
| `lexicon.formatOnSave` | `false` | `lex fmt` on save |
| `lexicon.trace.server` | `off` | Client tracing |

Keybindings (`.lex` files): `F5` run, `Ctrl+Shift+C` check,
`Ctrl+Shift+I` format, `Ctrl+Shift+V` vet.

## Project layout

```text
lexicon-vscode/
├── package.json                 # v1.0.0 super manifest
├── language-configuration.json  # brackets, indent, onEnter, folding
├── syntaxes/lexicon.tmLanguage.json
├── snippets/lexicon.json        # 85 snippets (source of truth)
├── SNIPPETS.md                  # generated catalog (do not hand-edit)
├── themes/lexicon-dark-pro.json # extended with Lexicon 1.0 scopes
├── themes/lexicon-midnight.json # extended
├── themes/lexicon-light.json    # NEW light theme
├── themes/lexicon-icon-theme.json
├── src/extension.ts + src/modules.ts + src/ambient.d.ts
├── out/extension.js + out/modules.js  # compiled with `tsc -p ./`
├── README.md / CHANGELOG.md
└── tsconfig.json
```

Rebuild host: `tsc -p ./` (global tsc works; `src/ambient.d.ts`
removes the `node_modules` requirement). Validate JSONs:
`python3 -c "import json; json.load(open('...'))"`.

## Autocomplete (context-aware + auto-import)

Typing `Http::` lists **only** Http members (`serve`, `get`, `post`).
Accepting one inserts the bare name (with `($1)` snippet) and, when
`import core::net::Http;` is missing, automatically adds the import
after your last import line. Works for every module (`Db::`, `Json::`,
`Env::`, `Console::`, `List::`, `Map::`, `Option::`, `Result::`,
`Channel::`) and for partial names (`Http::ge` → `get`).

On `import` lines you get module-path completion (`core::…`); anywhere
else you get keywords, builtin types, builtin functions
(`print/println/panic/recover/assert/inspect/spawn`) and modules.
Verified by 13 automated harness checks (see `AUTOCOMPLETE-ALL-PASS`).

## Comments & pre-run syntax gate

Commented-out code never counts: hover, go-to-definition, references,
rename, symbols, folding, semantic tokens and quick fixes all ignore
`//`, `/* */` and string contents (positions stay exact).

Before **every** run command, the extension pre-checks the file and
shows clear errors with line:col — `E0101` unterminated
string/char/block-comment, `E0201` unbalanced brackets or dotted
imports (`import a.b;` → use `a::b::C;`), plus `W0001`/`W0002`
warnings. Errors block with options (*Show Problems* / *Run Anyway*);
the same findings also appear on save via `lex vet`.

## Known limitations (binary-driven, honest)

- `lex check`/`lex build` in the shipped `lex.exe` predate current
  sources — the extension prefers `lex vet` for diagnostics.
- `Env::get` evaluates when assigned first (`let x = Env::get(..)`).
- `Http::serve` binds a literal address; `.env` files change behavior,
  not the port (see `demo-api/`).

## Autocomplete deep-dive

The completion provider (`src/extension.ts`) is context-aware with
three branches; the new provider split (other agent, `src/` providers)
keeps this behavior and only moves it:

1. `import core::ne|` → module-path completions (`core::net::Http`, …).
2. `Http::` / `Http::ge|` → **only** that module's members, replacing
   just the partial name; accepting one auto-inserts
   `import core::net::Http;` after the last import line when missing.
3. Anywhere else → keywords, builtin types, builtin functions
   (`print/println/panic/recover/assert/inspect/spawn`), modules (each
   with auto-import) and the `fn` snippet.

Hover covers keywords, decorators (`@Get/@Post/@Put/@Delete/@Test`),
`Module::member` signatures and local `fn`/type definitions with their
`///` doc comments.

## Debugging (v1 adapter: what works)

The `lexicon` debug type (`Lexicon Debug`) is a v1 adapter:

- Launch configs require only `program` (the file to debug; the
  **Lexicon: Launch file** snippet fills `${file}` automatically).
- Optional `args` (array) and `stopOnEntry` (boolean) are accepted and
  forwarded to the `lex debug` backend.
- Variables view is currently empty (`variables: []`) — inspection
  happens via the Debug Console / terminal output.

## Tasks + problemMatcher usage

`"type": "lex"` tasks run `run/build/test/check/vet/lint/bench`.
`taskDefinitions` carries no `problemMatcher` field by schema, so pair
tasks with the `lexicon-vet` matcher explicitly — it parses
`file.lex:line:column: message` lines plus `E0000`-style codes into
clickable Problems entries:

```json
{
  "label": "lex vet current file",
  "type": "lex",
  "command": "vet",
  "file": "${file}",
  "problemMatcher": ["lexicon-vet"]
}
```

## Walkthrough

The `lexicon.getStarted` walkthrough (3 steps: install → project →
run/test/debug) renders [WALKTHROUGH.md](./WALKTHROUGH.md). All three
steps point at that single file — the supported pattern, since step
media must be a file path, not inline markdown.

## i18n

Every user-facing manifest string lives in `package.nls.json`
(English defaults) and `package.nls.pt-br.json` (Português); the
manifest references them as `%key%`. Both files must define identical
key sets — the validation step below checks this.

## Web (vscode.dev notes)

The extension ships a browser entry (`./out/extension.web.js`, built by
the main flow from `src/extension.web.ts`). On vscode.dev there is no
local `lex` binary, so binary-backed commands (run/check/vet/lint/fmt/
doc/trace/test/build/debug/newProject) degrade to info messages;
editing, grammar, snippets, themes, walkthrough and i18n work fully.

## Extension Pack (roadmap)

A marketplace extension pack (themes + snippets + tooling bundles) is
deferred — no pack folders are created in this release.
