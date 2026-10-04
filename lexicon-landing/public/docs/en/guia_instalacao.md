# Installation and Setup - Lexicon v0.2.0

## 1. Prerequisites

- OS: Windows 10+, Linux (Ubuntu 20.04+), macOS (Catalina+).
- Disk: 500 MB free.
- Internet: only for the SDK download (the toolchain runs 100% locally).

## 2. Silent automatic install (recommended)

Every `lex` runs a **silent auto-install**: it copies the binary to
`~/.lexicon/bin` (only when changed) and adds it to PATH **only if absent**
(checking both the process env and the persistent store — no duplicates, no output).

```powershell
# Windows (PowerShell)
iwr https://get.lexicon.dev/install.ps1 -useb | iex
```

```bash
# Linux and macOS
curl -fsSL https://get.lexicon.dev | sh
```

To skip auto-install (CI/benchmarks):

```bash
CI=true lex run main.lex
# or
LEXICON_NO_AUTO_INSTALL=1 lex run main.lex
```

## 3. Manual install

```bash
lex install      # copy binary + register PATH (idempotent)
lex uninstall    # remove binary + clean empty ~/.lexicon
```

- If `~/.lexicon/bin` is already on PATH, `lex install` prints
  "already in your system PATH" and duplicates nothing.
- On Windows PATH is written to the user registry (`HKCU\Environment`);
  **reopen your terminal** so the new session sees the entry.

## 4. VS Code setup

Option A — super extension (recommended, v1.1.x):

1. Open VS Code → **Extensions** (Ctrl+Shift+X).
2. Find **Lexicon Super** (`lexicon-team.lexicon-super`).
3. Install, reload, open any `.lex` — grammar, 85+ snippets, 4 themes,
   contextual autocomplete, pre-run gate and vet-on-save work together.

Option B — no extension (toolchain scaffold):

```bash
lex ide init     # generates .vscode/tasks.json + snippets + LEX-TOOLS.md
```

This creates tasks for `lex run --ci`, `lex run --watch` (hot reload),
`lex test` and `lex check` on the current file, plus instructions to point
any generic LSP client at `lex lsp`.

## 5. Verify the installation

```bash
lex --version
```

The output should identify the **v0.2.0** language binary. Then validate
the toolchain for real:

```bash
lex vet demo-api\models.lex   # expect: "vet: no correctness violations"
lex check demo-api\main.lex   # expect: "Type checking passed!"
```

If `lex` is not recognized: reopen the terminal (persisted PATH needs a new
session) and run `lex install` again — it is idempotent and reports exactly
what was missing.
