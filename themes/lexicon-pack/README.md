# Lexicon Extension Pack

One install for the whole Lexicon VS Code experience. This pack contains:

- **lexicon-team.lexicon-super** — grammar (`.lex` / `.lang`), 85 snippets,
  Lexicon Dark Pro / Midnight / Light / High Contrast themes, toolchain
  commands (`run`, `check`, `vet`, `lint`, `fmt`, `test`, …), hover /
  completion / definition / symbols / CodeLens and vet-on-save diagnostics.

## Icon (copy step — the pack must stay self-contained)

`package.json` points at `./icons/lexicon-icon.png`, but the PNG lives in the
main extension (`../lexicon-vscode/icons/lexicon-icon.png`) and is intentionally
**not** duplicated in git. Before packaging, copy it in (PowerShell):

```powershell
New-Item -ItemType Directory -Path ".\icons" -Force
Copy-Item "..\lexicon-vscode\icons\lexicon-icon.png" ".\icons\lexicon-icon.png"
```

then:

```powershell
vsce package
```

## Versioning

Keep this pack's `version` in lockstep with `lexicon-super` releases so users
always get the latest bundled extension on update.
