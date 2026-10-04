# Lexicon walkthrough

This guide backs the **Get started with Lexicon** walkthrough
(`lexicon.getStarted`). Each walkthrough step renders this file, so every
section below is reachable from the VS Code *Get Started* experience.

## 1. Install the `lex` toolchain

1. Build `lex` from the repo root (linking needs MSVC on Windows;
   `cargo check` validates without linking).
2. Put the binary on your `PATH`, or point the extension at it:

```json
{
  "lexicon.path": "C:\\tools\\lex.exe"
}
```

3. Verify with **Lexicon: Show Toolchain Env** (status-bar `Lexicon` item).

Binary-backed commands (`run`, `check`, `vet`, `lint`, `fmt`, `doc`,
`trace`, `test`, `build`, `debug`) shell out to that binary. On
[vscode.dev](https://vscode.dev) there is no local binary, so those
commands degrade to info messages — editing, themes, snippets and the
walkthrough still work.

## 2. Create or open a project

- Run **Lexicon: New Project** (`lexicon.newProject`) and pick a name
  (default `my_lex_app`), or open any folder containing `.lex`/`.lang` files.
- Saving a `.lex` file runs `lex vet` automatically when
  `lexicon.vetOnSave` is `true` (default); findings appear in the
  **Problems** panel.
- Toggle auto-format with `lexicon.formatOnSave` (`lex fmt` on save).

## 3. Run, test, debug — and wire up tasks

- **Run**: `F5` (`Lexicon: Run File`) on the active `.lex` file, or
  `Lexicon: Run with Arguments` to append args after `--`.
- **Test**: `Lexicon: Run Tests` (`lex test`); `@Test` functions are
  picked up automatically. `main` functions also get Run/Test CodeLens.
- **Debug**: create a launch config with the **Lexicon Debug** debugger
  (`type: lexicon`). The snippet **Lexicon: Launch file** needs only
  `program` (defaults to the current file via `${file}`):

```json
{
  "type": "lexicon",
  "request": "launch",
  "name": "Launch",
  "program": "${file}"
}
```

- **Tasks**: `"type": "lex"` tasks (`run/build/test/check/vet/lint/bench`)
  pair with the `lexicon-vet` problem matcher so `file:line:column`
  output becomes clickable Problems entries:

```json
{
  "label": "lex vet current file",
  "type": "lex",
  "command": "vet",
  "file": "${file}",
  "problemMatcher": ["lexicon-vet"]
}
```

## 4. Next steps

- `Lexicon: Show Snippet Catalog` opens the 85-snippet reference.
- Typing `Module::` completes only that module's members and auto-inserts
  the missing `import core::…;` line (see README Autocomplete deep-dive).
- Four themes ship with the extension, including **Lexicon High Contrast**
  (`hc-black`); UI strings are localized in English and Portuguese
  (`package.nls*.json`).
