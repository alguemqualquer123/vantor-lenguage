# Lex Language (VS Code)

Syntax highlighting, snippets, tasks and problem matchers for the Lex
programming language.

## Requirements

Install the Lex toolchain and put `lex` on `PATH`:

```sh
lex sdk info
```

## Features

- TextMate grammar for `*.lex` (keywords, strings with interpolation,
  decorators like `@Get`/`@Test`, module paths, numeric suffixes)
- 20+ snippets: `main`, `fn`, `import std`, `lambda`, `assert`,
  `heap::New`, `sha256::Sum`, HTTP endpoints, …
- `lex` task type (run/build/test/check/vet/lint/fmt/bench/serve)
  with `$lexicon-*` problem matchers
- Language configuration: brackets, comments, indentation

## Language server

Point any generic LSP client at the bundled binary:

```
lex lsp
```

`lex complete --file main.lex --line 3 --col 11 --json` answers from the
same engine (60+ modules: 13 native builtins + 45 `std::` packages).

## Standard library

`lib/std/**/*.lex` in the SDK: strings/strconv/math/sort/slices/fmt,
time/bytes/io/bufio/maps/cmp/iter, containers, os/sync/context/log,
net/url, encodings, regexp, hashes, rand, crypto.
