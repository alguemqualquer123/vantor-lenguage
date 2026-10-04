# Installation and Setup — Lexicon v0.3.5

Everything below was run against the `0.3.5` toolchain. If a command is not in
this page, it does not exist.

## 1. Prerequisites

- Windows 10/11 x64 (pre-built binary), **or** Linux/macOS with Rust (`cargo`)
  and `git` to compile from source.
- ~500 MB of free disk. The toolchain is **one 9.4 MB binary**: no runtime,
  JVM or Node to install.
- Internet only for the download. After that `lex` runs 100% locally.

## 2. What there is to download

The artifacts live in the **release assets** of
[`alguemqualquer123/vantor-lenguage`](https://github.com/alguemqualquer123/vantor-lenguage/releases).
`.../releases/latest/download/<file>` always points at the newest version.

| Asset | Contents | Size |
|---|---|---|
| `lex-sdk-0.3.5-windows-x64.zip` | Full SDK: `bin/lex.exe` + 38 shims, `lib/std` (89 packages), `examples` (19), `templates` (5), `docs`, `scripts`, `VERSION` | 4.4 MB zip / 9.7 MB unpacked |
| `lex-0.3.5-windows-x64.zip` | Just the language: one `lex.exe` | 4.3 MB zip / 9.4 MB |
| `install.ps1` | Windows installer | 3 KB |
| `install.sh` | Linux/macOS installer (downloads the platform asset when one exists, otherwise compiles from source) | 2.6 KB |
| `lexicon-super-1.2.0.vsix` | VS Code super extension | 247 KB |
| `checksums.txt` | SHA-256 of every asset above | — |

Both zips give you **the same toolchain**. The difference: the SDK zip already
ships the stdlib, examples and templates unpacked, while the "just the
language" zip is a single file that writes that same SDK into
`~/.lexicon/sdk` on its first run.

## 3. Script install (recommended)

```powershell
# Windows — PowerShell, no admin needed
powershell -ExecutionPolicy Bypass -Command "iwr -useb https://github.com/alguemqualquer123/vantor-lenguage/releases/latest/download/install.ps1 | iex"
```

```bash
# Linux and macOS
curl -fsSL https://github.com/alguemqualquer123/vantor-lenguage/releases/latest/download/install.sh | sh
```

The script downloads, unpacks into a temp folder and calls `lex install`,
which:

1. copies the binary to `~/.lexicon/bin` (`.lexicon\bin` on Windows);
2. creates the 38 launchers (`lex-run`, `lex-check`, `lex-mod`, …) — ~40-byte
   shims that forward the subcommand to the neighbouring binary;
3. adds `~/.lexicon/bin` to PATH **only if it is missing** (it checks both the
   process env and the persistent store — no duplicates);
4. exports the SDK into `~/.lexicon/sdk`.

## 4. Manual install

1. Download and unzip wherever you like.
2. Run `lex.exe install` (or `./lex install`).
3. **Open a new terminal** — a PATH entry written to the registry only applies
   to new sessions.
4. Verify: `lex version`.

## 5. Silent auto-install

Every `lex` run performs the same reconciler as section 3, silently: it copies
the binary when it changed, keeps the PATH entry, and refreshes the SDK when
`~/.lexicon/sdk/VERSION` differs from the running toolchain. No output, no
duplicates.

To skip it (CI, benchmarks, timing runs):

```bash
CI=true lex run main.lex
# or
LEXICON_NO_AUTO_INSTALL=1 lex run main.lex
```

## 6. Verifying the installation

```bash
lex version        # lex 0.3.5 (lexc 0.3.5, spec v0.1)
lex sdk verify     # check the required files of the installed SDK
lex sdk info       # flavour, binary path and binary size
```

`lex sdk verify` prints `SDK verify OK at <dir> (116 files)` when the kit is
complete, and fails if any package, example or template goes missing.

Now run something for real (on Windows the installed SDK is at
`%USERPROFILE%\.lexicon\sdk`, elsewhere at `~/.lexicon/sdk`):

```bash
lex run <sdk>/examples/fizzbuzz.lex
lex vet <sdk>/examples/fizzbuzz.lex   # expect: no correctness violations
lex check <sdk>/examples/pong.lex     # expect: Type checking passed!
```

## 7. VS Code

Option A — super extension (`lexicon-team.lexicon-super` 1.2.0):

```bash
code --install-extension lexicon-super-1.2.0.vsix
```

It brings the grammar, 117 snippets, 5 themes, contextual autocomplete with
auto-import and `lex vet` diagnostics on save.

Option B — no extension, toolchain scaffold:

```bash
lex ide init     # generates .vscode/tasks.json + snippets + LEX-TOOLS.md
```

That creates tasks for `lex run --ci`, `lex run --watch` (hot reload),
`lex test` and `lex check` on the current file, and documents how to point any
generic LSP client at `lex lsp` (or the debugger at `lex dap`).

## 8. Building from source (and shipping your own release)

```bash
git clone https://github.com/alguemqualquer123/vantor-lenguage.git
cd vantor-lenguage
cargo build --profile dist -p lexicon-cli     # opt-level=z + fat LTO
./target/dist/lex install                      # PATH + SDK
```

To package the same artifacts this page offers:

```powershell
powershell -ExecutionPolicy Bypass -File release.ps1
```

It builds, runs `lex sdk export` and `lex sdk verify`, zips both flavours and
writes `checksums.txt` into `build/release/`.

## 9. Removing it

```bash
lex uninstall    # removes the binary, the launchers and the PATH entry
```

Reopen your terminal afterwards. Windows PATH is written to
`HKCU\Environment`; `lex install` is idempotent and reports exactly what was
missing when you run it again.
