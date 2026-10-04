# Lexicon Programming Language (v0.3.4)

Lexicon is a modern programming language with high-level syntax and low-level
performance — one `lex` binary that carries the compiler, interpreter,
89-package standard library, package manager, language server and the wgpu
graphics engine.

> Language version follows the workspace `Cargo.toml` (**0.3.x**). The VS Code
> extension is versioned separately (**1.1.x**) — do not confuse the two.
> The playful, mascot-powered introduction lives in the repository root
> [`README.md`](../README.md); release notes are in
> [`CHANGELOG.md`](../CHANGELOG.md).

## Running Lexicon

Invoke the toolchain as `lex <command> [args]` — either the built binary
(`target/dist/lex.exe`, `target/release/lex.exe`) or any SDK launcher
(`lex-run`, `lex-check`, `lex-mod`, …), which are ~40-byte shims that forward
to the single `lex` binary next to them.

> The old `lex.bat` / `lex.sh` prefix wrappers are **gone**; `build.bat` and
> `build.sh` still drive `cargo build`.

### Common Commands
- `lex new <project_name>` - Create a new project (`default`, `api`, `plugin`, `service` templates).
- `lex run <file.lex>` - Run a Lexicon file.
- `lex run --watch <file.lex>` - Run with hot reload (save a `.lex` file to auto-restart; 300 ms debounce, `.lex`-only filter, crash backoff).
- `lex run --ci <file.lex>` - Non-blocking run: GUI windows auto-quit after ~1 s.
- `lex check <file.lex>` - Type-check without building.
- `lex vet <file.lex>` - Static correctness checks (+ advisory security findings).
- `lex lint` / `fmt` / `doc` / `trace` - Lint, format, API docs, execution trace (all comment-accurate).
- `lex visualize <file.lex>` - Visualize data flow in pipes.
- `lex repl` - Start the interactive shell.
- `lex test` / `lex bench` - Run the test suite and the benchmarks.
- `lex mod init|add|install|list|graph|verify` - Package manager (`lexicon.toml` + `lexicon.lock` + `lex_packages/`).
- `lex gui` - Run a Lexicon GUI application (native eframe + wgpu).
- `lex lsp` / `lex dap` / `lex ide` - Language server, debug adapter, editor scaffolding.
- `lex sdk export` / `lex sdk verify` - Build and check the SDK kit (one binary + launchers, size budget enforced).
- `lex install` / `lex uninstall` - Put `lex` on the system PATH.
- `lex version` - Show versions.

## Where to read next

| Document | Contents |
|---|---|
| [`../README.md`](../README.md) | Language tour, mascot, quick start |
| [`PLANO_GO_FULL_PARITY.md`](PLANO_GO_FULL_PARITY.md) | Go-parity plan, phase by phase |
| [`FEATURE_MATRIX.md`](FEATURE_MATRIX.md) | Cargo feature gates and binary budgets |
| [`missing_implementations.md`](missing_implementations.md) | What is genuinely still missing, and why |
| [`../CHANGELOG.md`](../CHANGELOG.md) | Release notes |
| [`../TODO.md`](../TODO.md) | Spec-conformance checklist (source of truth) |
| [`../LEX-TOOLS.md`](../LEX-TOOLS.md) | Every CLI subcommand in detail |
