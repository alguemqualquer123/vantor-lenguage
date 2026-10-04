# Changelog — Lexicon Super Extension

## 1.2.0 – colors, icon & SDK go-to-definition fixes

- **Syntax colors restored.** The workspace associated `*.lex` with the
  language id `lex`, but every contribution (grammar, icon, providers,
  `onLanguage` activation) is registered under `lexicon`. The mismatch
  meant the TextMate grammar never applied and the extension never
  activated for `.lex` files. Fixed the association to `lexicon` (and
  `*.lang`), so highlighting, semantic tokens and all providers now load.
- **File/language icon fixed.** The icon theme referenced its icons as
  `./icons/…`, which resolves relative to `themes/` (a folder that does
  not exist) — every icon was broken. Repointed all 27 definitions to
  `../icons/…`, fixed `languageIds.lexicon` (`lex` → `_lex`) and dropped
  the dangling `lock` → `_lock` mapping. New `test/validate-icons.js`
  fails the build on any dangling reference or missing file.
- **Ctrl+Click → SDK (Go-style GOROOT discovery).** `sdkRoots()` now
  resolves the standard library from, in order: the new `lexicon.sdkPath`
  setting, `LEXICON_HOME`/`LEXICON_SDK`/`LEX_PATH` env vars, the absolute
  `lex` binary location (config → `~/.lexicon/bin` → `~/.lexicon/sdk/bin`
  → PATH lookup via `where`/`which`), `~/.lexicon/sdk`, machine-wide
  install dirs (`C:\Program Files\Lexicon`, `/usr/local/lexicon`, …) and
  the workspace. `Lexicon: Show Toolchain Env` now prints the resolved
  binary and every candidate SDK root so detection is verifiable.
- **Semantic tokenizer stress fix.** The overlap guard scanned every
  prior token in the file (O(n²)); tokens only ever overlap within a
  line, so the guard is reset per line. A 4000-line / 396 KB file now
  tokenizes without the quadratic blow-up.
- **Release tooling.** `scripts/release.js` bumps the version, compiles,
  runs the full test suite (icon validation, grammar tokenization, an
  extension load test that activates against a mocked `vscode` API and
  exercises the definition + semantic-token providers), then packages a
  dependency-free `.vsix`.
- Tests: `test/tokenize.js` (91% scope coverage on a feature sample),
  `test/validate-icons.js`, `test/load-extension.js` (all green).

## 1.1.1 – comments & pre-run syntax gate

- `lexicon-lexer/src/comments.rs` (new): `strip_comments`/`code_contains`
  mirroring the tokenizer (8 unit tests green); runner, lint and
  security scan now operate on comment-free code – commented-out
  `print`, `Http::serve` and ghost routes never execute or register.
- Extension: `maskNonCode` + `precheckLexicon` – E0101/E0201 with exact
  line:col, W0001/W0002 warnings; pre-run gate on every run command
  (*Show Problems* / *Run Anyway*); same findings merged into
  save-diagnostics; hover/definition/references/rename/symbols/folding/
  semantic/quickfix all ignore comments and strings.
- New `/* */` block-comment snippet (86 total, catalog regenerated).
- 20-check Node harness green (`PRECHECK-ALL-PASS`).

## 1.1.0 — i18n, debugger, problem matcher, walkthrough, high-contrast, web

- Manifest (`package.json` 1.1.0): new top-level `browser` entry
  (`./out/extension.web.js`, built by the main flow); new
  `contributes.debuggers` (`type: lexicon`, `Lexicon Debug`, launch
  requires `program`, optional `args`/`stopOnEntry`, `Lexicon: Launch
  file` snippet defaulting to `${file}`); new
  `contributes.problemMatchers` (`lexicon-vet`: `file.lex:line:col`
  + `E0000` patterns, pairs with `"type": "lex"` tasks); new
  `contributes.walkthroughs` (`lexicon.getStarted`, 3 steps rendering
  `WALKTHROUGH.md`); missing `onCommand` activation events filled in.
- i18n: `package.nls.json` (en) + `package.nls.pt-br.json` (pt-BR);
  every user-facing manifest string is a `%key%` reference with
  identical key sets in both files.
- Settings docs: every `lexicon.*` property gained a
  `markdownDescription` with a copy-paste JSON code example
  (`path/features/target/vetOnSave/formatOnSave/trace.server`).
- Themes: NEW `Lexicon High Contrast` (`hc-black`: pure black
  background, white/yellow/cyan/magenta token colors covering all 98
  Lexicon 1.0 grammar scopes); `editorBracketHighlight.foreground1..6`
  + `unexpectedBracket.foreground` added to Dark Pro, Midnight and
  Light; `semanticTokenColors`
  (function/variable/parameter/property/namespace/type) added to all 4
  themes.
- Docs: README gained Autocomplete deep-dive, Debugging (v1 adapter),
  Tasks+problemMatcher, Walkthrough, i18n, Web (vscode.dev binary
  degradation) and Extension Pack (deferred) sections; new
  `WALKTHROUGH.md` guide.
- `language-configuration.json` untouched (owned by the main flow).

## 1.0.1 — contextual autocomplete + auto-import

- Completion is now context-aware: `Module::` lists only that
  module's members (filtered as you type, bare-name snippet insert);
  `import` lines complete `core::…` paths; unknown modules yield nothing.
- Automatic imports: accepting a member/module completion inserts
  `import core::x::Y;` after the last import (skipped when present).
- Fixed stdlib paths to real `::` syntax (`core::net::Http`, …) —
  the dotted form produced invalid code.
- Fixed `vscode.TextEdit.insert` factory misuse (was `new`-ed).
- Added builtin-function completions
  (`print/println/panic/recover/assert/inspect/spawn`).
- 13-check Node harness green (`AUTOCOMPLETE-ALL-PASS`).

## 1.0.0 — super extension

### Grammar (`syntaxes/lexicon.tmLanguage.json`)
- Rewritten from scratch: the previous file had duplicated repository
  content appended and was **invalid JSON**.
- Faithful to `lexicon-lexer/src/tokens.rs`: `switch/case/default/do`,
  `defer`, `panic/recover`, `spawn/select/channel/actor/task`,
  `unsafe/extern/native/inline`, `int/uint/byte/rune/decimal`,
  hex/binary/float+exponent literals, `{var}` interpolation (real
  syntax, not `${}`), `///` docs, `@Get/@Post/@Put/@Delete/@Test`,
  `#[cfg(...)]`, `|>` pipes, `|x|` closures, generics, `::` paths.
- All regexes compile; every `#include` resolves.

### Snippets (`snippets/lexicon.json`)
- Rewritten: 85 Lexicon-accurate snippets (were Rust-flavored:
  `impl Trait for`, `mod`, `use`, `lazy_static!` removed/replaced).
- New: `api`, `serve`, `dbcon/dbq/dbx/dbi`, `printenv`, `switch`,
  `matchg`, `mbool`, `defer`, `vfn`, `cfg`, `dowhile`, `slit`, `classi`.
- `SNIPPETS.md` catalog generated from the JSON (single source of truth).

### Themes
- Dark Pro: +31 `Lexicon 1.0` token rules for the new grammar scopes.
- Midnight: workbench kept, tokenColors extended (+18 Lexicon rules).
- NEW: `Lexicon Light` full theme (workbench + 30 token rules).
- Icon theme untouched.

### Language configuration
- Added `indentationRules`, `onEnterRules` (`///` continuation,
  `/** */`, brace indent-outdent), `wordPattern`, `#region` folding.

### Extension host (`src/` → `out/`, compiled with global `tsc`)
- Removed dead root `extension.js` (features merged into `src/`).
- New: 14 commands (run/check/vet/lint/fmt/doc/trace/test/build/debug/
  newProject/showSnippets/env/runWithArgs), DocumentSymbolProvider,
  Run/Test CodeLens, vet-on-save diagnostics, status bar, output
  channel, settings-driven `--features`/`--target`/binary path.
- `src/ambient.d.ts` allows `tsc -p ./` with zero `node_modules`.
- `src/modules.ts` stdlib data aligned with the real runtime
  (`Http::serve/get/post`, `Json`, `Env::get`, `Db::connect/query/execute`).

### Manifest (`package.json`)
- Renamed `lexicon-super` 1.0.0; `.lang` support; task definitions
  (`"type": "lex"`); editor/context menus; 5 keybindings; real
  `configuration` schema (`lexicon.path/features/target/vetOnSave/
  formatOnSave/trace.server`); Light theme + icon theme kept.

## 0.2.0 (previous)
- Baseline: grammar stub, Rust-style snippets, 2 dark themes,
  run/debug/newProject/build commands, hover/completion/definition.
