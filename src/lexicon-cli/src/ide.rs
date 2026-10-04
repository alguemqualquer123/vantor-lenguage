//! Editor integration (`lex ide init [dir]`, `lex ide extension [dir]`).
//!
//! Scaffolds VSCode assets that work with zero extensions:
//! - `.vscode/tasks.json` — Run / Watch / Test / Check (Ctrl+Shift+B).
//! - `.vscode/lex.code-snippets` — auto-loaded snippets (`fn main`,
//!   `@Get` endpoint, `match`, `import std`, `lambda`, `assert`,
//!   per-package imports, `heap::New`, `sha256::Sum` …).
//! - `.vscode/settings.json` — `*.lex` mapped to the real `lex` language.
//! - `LEX-TOOLS.md` — how to point any generic LSP client at `lex lsp`
//!   (initialize/completion/hover over stdio) and how to use
//!   `lex complete --file … --line … --col …` from scripts.
//!
//! `lex ide extension [dir]` scaffolds the zero-build `vscode-lex`
//! extension: TextMate grammar (`source.lex`), language configuration,
//! the same snippets, the `lex` task type and `$lexicon-*` matchers.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const TASKS_JSON: &str = r#"{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "lex: build (debug)",
      "type": "lex",
      "command": "build",
      "file": "${file}",
      "group": { "kind": "build", "isDefault": true },
      "problemMatcher": ["$lexicon-build"],
      "presentation": { "echo": true, "reveal": "silent", "panel": "shared" }
    },
    {
      "label": "lex: build --release",
      "type": "lex",
      "command": "build",
      "file": "${file}",
      "release": true,
      "group": "build",
      "problemMatcher": ["$lexicon-build"]
    },
    {
      "label": "lex: run current file",
      "type": "lex",
      "command": "run",
      "file": "${file}",
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "lex: watch current file (hot reload)",
      "type": "lex",
      "command": "run",
      "file": "${file}",
      "watch": true,
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "lex: test",
      "type": "lex",
      "command": "test",
      "group": { "kind": "test", "isDefault": true },
      "problemMatcher": ["$lexicon-test"],
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "lex: check current file",
      "type": "lex",
      "command": "check",
      "file": "${file}",
      "problemMatcher": ["$lexicon-check"],
      "presentation": { "reveal": "silent", "panel": "shared" }
    },
    {
      "label": "lex: vet current file",
      "type": "lex",
      "command": "vet",
      "file": "${file}",
      "problemMatcher": ["$lexicon-vet"]
    },
    {
      "label": "lex: lint",
      "type": "lex",
      "command": "lint",
      "problemMatcher": ["$lexicon-check"]
    },
    {
      "label": "lex: fmt --check",
      "type": "lex",
      "command": "fmt",
      "file": "${file}",
      "args": ["--check"],
      "problemMatcher": []
    },
    {
      "label": "lex: bench",
      "type": "lex",
      "command": "bench",
      "problemMatcher": ["$lexicon-test"]
    },
    {
      "label": "lex: serve (background)",
      "type": "lex",
      "command": "serve",
      "file": "${file}",
      "host": "127.0.0.1",
      "port": 3000,
      "isBackground": true,
      "problemMatcher": {
        "pattern": { "regexp": "^.*$" },
        "background": {
          "activeOnStart": true,
          "beginsPattern": ".*(listening|serving|running).*",
          "endsPattern": ".*(error|failed).*"
        }
      },
      "presentation": { "reveal": "always", "panel": "dedicated" }
    },
    {
      "label": "lex: test-net",
      "type": "lex",
      "command": "test-net",
      "host": "127.0.0.1",
      "port": 3000
    },
    {
      "label": "lex: dns-check",
      "type": "lex",
      "command": "dns-check",
      "host": "example.com",
      "args": ["--timeout", "5s"]
    },
    {
      "label": "lex: tls-check",
      "type": "lex",
      "command": "tls-check",
      "host": "example.com",
      "port": 443
    }
  ]
}
"#;

const SETTINGS_JSON: &str = r#"{
  "files.associations": {
    "*.lex": "lex"
  },
  "editor.quickSuggestions": {
    "other": true,
    "comments": false,
    "strings": true
  },
  "editor.acceptSuggestionOnCommitCharacter": true,
  "editor.tabCompletion": "on"
}
"#;

const SNIPPETS_JSON: &str = r#"{
  "Lex main": {
    "prefix": ["main", "fn main"],
    "body": ["pub fn main() -> void {", "\t$0", "}"],
    "description": "Lex program entry point"
  },
  "Lex function": {
    "prefix": ["fn"],
    "body": ["pub fn ${1:name}() -> ${2:void} {", "\t$0", "}"],
    "description": "Lex function declaration"
  },
  "Lex HTTP endpoint": {
    "prefix": ["@Get", "endpoint", "route"],
    "body": [
      "@${1|Get,Post,Put,Delete|}(\"${2:/}\")",
      "pub fn ${3:handler}() -> String {",
      "\treturn \"${4:ok}\";",
      "}",
      "",
      "pub fn main() -> void {",
      "\tHttp::serve(\"0.0.0.0:3000\");",
      "}"
    ],
    "description": "Lex HTTP endpoint served by the real server"
  },
  "Lex serve": {
    "prefix": ["serve", "Http::serve"],
    "body": ["Http::serve(\"0.0.0.0:${1:3000}\");"],
    "description": "Start the real Lex HTTP server"
  },
  "Lex match": {
    "prefix": ["match"],
    "body": ["match ${1:x} {", "\t$0", "}"],
    "description": "Exhaustive pattern match"
  },
  "Lex struct": {
    "prefix": ["struct"],
    "body": ["struct ${1:Name} {", "\t$0", "}"],
    "description": "Lex struct declaration"
  },
  "Lex test": {
    "prefix": ["@Test", "test"],
    "body": ["@Test", "pub fn ${1:test_}() -> bool {", "\treturn $0;", "}"],
    "description": "Test discovered by lex test"
  },
  "Lex Http get": {
    "prefix": ["Http::get", "fetch"],
    "body": ["Http::get(\"${1:https://…}\")"],
    "description": "Blocking HTTP GET at runtime"
  },
  "Lex Json parse": {
    "prefix": ["Json::parse"],
    "body": ["Json::parse(\"${1:{…}}\")"],
    "description": "Parse JSON at runtime"
  },
  "Lex print": {
    "prefix": ["print", "Console::writeLine"],
    "body": ["Console::writeLine(\"${1:…}\");"],
    "description": "Print a line to stdout"
  },
  "Lex import std": {
    "prefix": ["import std", "use std"],
    "body": ["import std::${1:strings};"],
    "description": "Import a standard-library package (key = last segment)"
  },
  "Lex lambda": {
    "prefix": ["lambda", "|x|"],
    "body": ["|${1:x}| ${2:x}"],
    "description": "First-class lambda (captures environment)"
  },
  "Lex assert": {
    "prefix": ["assert"],
    "body": ["assert(${1:cond}, \"${2:msg}\");"],
    "description": "Runtime assertion (fails the run)"
  },
  "Lex let": {
    "prefix": ["let"],
    "body": ["let ${1:name} = ${2:value};"],
    "description": "Immutable binding (also var, const)"
  },
  "Lex for": {
    "prefix": ["for"],
    "body": ["for ${1:x} in ${2:xs} {", "\t$0", "}"],
    "description": "Iteration (also while, loop)"
  },
  "Lex errors::New": {
    "prefix": ["errors::New", "new error"],
    "body": ["import std::errors;", "", "errors::New(\"${1:…}\")"],
    "description": "Go-parity error value"
  },
  "Lex strings import": {
    "prefix": ["import strings"],
    "body": ["import std::strings;"],
    "description": "Go-parity string helpers"
  },
  "Lex math import": {
    "prefix": ["import math"],
    "body": ["import std::math;"],
    "description": "Native float64 math + Pi/E consts"
  },
  "Lex json roundtrip": {
    "prefix": ["json::", "Marshal"],
    "body": ["import std::encoding::json;", "", "Json::serialize(${1:value})"],
    "description": "Native JSON serialize (Json::parse/valid)"
  },
  "Lex time now": {
    "prefix": ["time::Now", "Now()"],
    "body": ["import std::time;", "", "time::Now()"],
    "description": "Current instant (Format/ParseDate/Sleep)"
  },
  "Lex heap": {
    "prefix": ["heap::New"],
    "body": ["import std::container::heap;", "", "let ${1:h} = heap::New(|a, b| a < b);"],
    "description": "Min-heap with lambda comparator"
  },
  "Lex regexp match": {
    "prefix": ["regexp::Match"],
    "body": ["import std::regexp;", "", "regexp::Match(\"${1:pattern}\", ${2:s})"],
    "description": "Documented-subset regex match"
  },
  "Lex sha256": {
    "prefix": ["sha256::Sum"],
    "body": ["import std::crypto::sha256;", "", "sha256::Sum(${1:s})"],
    "description": "SHA-256 hex digest (FIPS vectors)"
  }
}
"#;

// Real TextMate grammar for Lex (replaces the old rust-approximation:
// `*.lex` now maps to the `lex` language id). Shipped inside the
// `editors/vscode-lex` extension; also used by every editor with
// TextMate support.
const GRAMMAR_JSON: &str = r##"{
  "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
  "name": "Lex",
  "scopeName": "source.lex",
  "fileTypes": ["lex"],
  "patterns": [
    { "include": "#comments" },
    { "include": "#strings" },
    { "include": "#decorators" },
    { "include": "#keywords" },
    { "include": "#declarations" },
    { "include": "#numbers" },
    { "include": "#module-paths" },
    { "include": "#types" }
  ],
  "repository": {
    "comments": {
      "patterns": [
        { "name": "comment.line.double-slash.lex", "match": "//.*$" },
        { "name": "comment.block.lex", "begin": "/\\*", "end": "\\*/" }
      ]
    },
    "strings": {
      "patterns": [
        {
          "name": "string.quoted.double.lex",
          "begin": "\"",
          "end": "\"",
          "patterns": [
            { "name": "constant.character.escape.lex", "match": "\\\\(\\\\|\"|n|t|r|0)" },
            { "name": "meta.interpolation.lex", "match": "\\{[^\\}]*\\}" }
          ]
        }
      ]
    },
    "decorators": {
      "patterns": [
        { "name": "entity.name.function.decorator.lex", "match": "@[A-Za-z_][A-Za-z0-9_]*" }
      ]
    },
    "keywords": {
      "patterns": [
        { "name": "keyword.control.lex", "match": "\\b(fn|return|if|else|match|for|while|loop|break|continue|defer|in|switch|case|default|do|throw|try|catch|finally|with|yield|await|async|spawn|select|task)\\b" },
        { "name": "keyword.declaration.lex", "match": "\\b(let|var|const|static|mut|pub|private|protected|struct|enum|class|interface|trait|impl|type|import|module|service|new|as|is|where|ref|self|super|crate|extern|unsafe|native|macro|derive|actor|effect)\\b" },
        { "name": "constant.language.lex", "match": "\\b(true|false|null|None|Some|Self)\\b" }
      ]
    },
    "declarations": {
      "patterns": [
        { "name": "meta.function.lex", "begin": "\\b(fn)\\s+([A-Za-z_][A-Za-z0-9_]*)", "beginCaptures": { "1": { "name": "keyword.declaration.lex" }, "2": { "name": "entity.name.function.lex" } }, "end": "(?=\\(|$)", "patterns": [{ "include": "#comments" }] },
        { "name": "meta.type.lex", "match": "\\b(struct|enum|class|interface|trait|type)\\s+([A-Za-z_][A-Za-z0-9_]*)", "captures": { "1": { "name": "keyword.declaration.lex" }, "2": { "name": "entity.name.type.lex" } } }
      ]
    },
    "numbers": {
      "patterns": [
        { "name": "constant.numeric.float.lex", "match": "\\b\\d[\\d_]*\\.\\d[\\d_]*(f32|f64)?\\b" },
        { "name": "constant.numeric.integer.lex", "match": "\\b\\d[\\d_]*(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128)?\\b" }
      ]
    },
    "module-paths": {
      "patterns": [
        { "name": "entity.name.namespace.lex", "match": "\\b(std|[A-Z][A-Za-z0-9_]*)::" },
        { "name": "support.function.lex", "match": "(?:::|\\.)[a-z_][A-Za-z0-9_]*(?=\\()" }
      ]
    },
    "types": {
      "patterns": [
        { "name": "entity.name.type.lex", "match": "\\b[A-Z][A-Za-z0-9_]*\\b" },
        { "name": "storage.type.lex", "match": "\\b(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|bool|char|String|Dynamic|void)\\b" }
      ]
    }
  }
}
"##;

const LANG_CONFIG_JSON: &str = r#"{
  "comments": { "lineComment": "//", "blockComment": ["/*", "*/"] },
  "brackets": [["{", "}"], ["[", "]"], ["(", ")"]],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" },
    { "open": "\"", "close": "\"", "notIn": ["string"] }
  ],
  "surroundingPairs": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
    ["\"", "\""]
  ],
  "wordPattern": "(-?\\d*\\.\\d\\w*)|([^\\`\\~\\!\\@\\#\\%\\^\\&\\*\\(\\)\\-\\=\\+\\[\\{\\]\\}\\\\\\|\\;\\:\\'\\\"\\,\\.\\<\\>\\/\\?\\s]+)",
  "indentationRules": {
    "increaseIndentPattern": "\\{[^}\"']*$",
    "decreaseIndentPattern": "^\\s*\\}"
  }
}
"#;

// Minimal installable VS Code extension (`editors/vscode-lex`). Zero
// build: grammar + language configuration + snippets + task definitions
// with problem matchers. The language server stays in the `lex` binary
// (`lex lsp` over stdio — wire any generic LSP client to it).
// NOTE: the extension versions independently from the toolchain
// (1.1.x line; toolchain is 0.3.0).
const EXT_PACKAGE_JSON: &str = r#"{
  "name": "vscode-lex",
  "displayName": "Lex Language",
  "description": "Syntax highlighting, snippets, tasks and problem matchers for the Lex programming language (toolchain: `lex`).",
  "version": "1.1.2",
  "publisher": "lexiconlang",
  "license": "MIT",
  "engines": { "vscode": "^1.85.0" },
  "categories": ["Programming Languages", "Snippets", "Linters"],
  "keywords": ["lex", "lexlang", "compiler", "lsp"],
  "contributes": {
    "languages": [
      {
        "id": "lex",
        "aliases": ["Lex", "lexlang"],
        "extensions": [".lex"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "lex",
        "scopeName": "source.lex",
        "path": "./syntaxes/lex.tmLanguage.json"
      }
    ],
    "snippets": [
      { "language": "lex", "path": "./snippets/lex.code-snippets" }
    ],
    "taskDefinitions": [
      {
        "type": "lex",
        "required": ["command"],
        "properties": {
          "command": { "type": "string", "description": "lex subcommand: run, build, test, check, vet, lint, fmt, bench, serve" },
          "file": { "type": "string", "description": "Target .lex file" },
          "release": { "type": "boolean", "description": "Release build" },
          "watch": { "type": "boolean", "description": "Hot reload (run only)" },
          "host": { "type": "string", "description": "Bind host (serve/test-net)" },
          "port": { "type": "number", "description": "Port (serve/test-net)" },
          "args": { "type": "array", "description": "Extra CLI args" }
        }
      }
    ],
    "problemMatchers": [
      {
        "name": "$lexicon-build",
        "owner": "lex",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": { "regexp": "^(.*):(\\d+):(\\d+):\\s+(error|warning):\\s+(.*)$", "file": 1, "line": 2, "column": 3, "severity": 4, "message": 5 }
      },
      {
        "name": "$lexicon-check",
        "owner": "lex",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": { "regexp": "^\\[(E\\d+)\\]\\s+(.*?)\\s+at\\s+(.*):(\\d+):(\\d+)$", "code": 1, "message": 2, "file": 3, "line": 4, "column": 5 }
      },
      {
        "name": "$lexicon-test",
        "owner": "lex-test",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": { "regexp": "^(PASS|FAIL|ok|FAILED)\\s+(.*)$", "severity": 1, "message": 2 }
      },
      {
        "name": "$lexicon-vet",
        "owner": "lex",
        "fileLocation": ["relative", "${workspaceFolder}"],
        "pattern": { "regexp": "^(.*):(\\d+):(\\d+):\\s+(.*)$", "file": 1, "line": 2, "column": 3, "message": 4 }
      }
    ]
  }
}
"#;

const EXT_README: &str = r#"# Lex Language (VS Code)

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
"#;

const TOOLS_MD: &str = r#"# Lex editor tools (generated by `lex ide init`)
## Out of the box (no extensions)
- **Tasks** (`.vscode/tasks.json`, type `lex`): `run` (+`--watch`
  hot reload), `build` (default, Ctrl+Shift+B), `test` (default),
  `check`, `vet`, `lint`, `fmt --check`, `bench`, `serve` (background
  on 127.0.0.1:3000), `test-net`, `dns-check`, `tls-check`.
- **Snippets** (`.vscode/lex.code-snippets`): `main`, `fn`, `@Get`,
  `serve`, `match`, `struct`, `@Test`, `Http::get`, `Json::parse`,
  `print`, plus `import std`, `lambda`, `assert`, `let`, `for`,
  `errors::New`, per-package imports (`import strings`, `import math`),
  `json::`, `time::Now`, `heap::New`, `regexp::Match`, `sha256::Sum`.
- `*.lex` files use the real Lex grammar (`source.lex`) once the
  `vscode-lex` extension below is installed (fallback: Rust approximation).

## Full extension (`lex ide extension [dir]`)

Scaffolds `editors/vscode-lex` (also committed in this repo): a zero-build
VS Code extension with the TextMate grammar (`syntaxes/lex.tmLanguage.json`),
language configuration, the same snippets, the `lex` task type and the
`$lexicon-build/check/test/vet` problem matchers. Install:

```sh
cd editors/vscode-lex && code --install-extension .
# or: vsce package && code --install-extension vscode-lex-*.vsix
```

## Autocomplete engine (`lex complete`)
`lex complete` answers from the same registry as the LSP — 75 modules:
14 native builtins (`Console::`, `Json::`, `Env::`, `Http::`, `Time::`,
`File::`, `Process::`, `Text::`, `Math::`, `List::`, `Hash::`, `Rand::`,
`Atomic::`, `Sys::`) and 61 `std::` packages (`strings::ToUpper`,
`sha256::Sum`, `url::Parse`, `heap::New`, …), every entry mirroring
something the toolchain honours (`lex complete --prefix "strings::"`
lists real `pub fn`s from `lib/std/strings.lex`):

```sh
lex complete --prefix "Http::"        # members of Http
lex complete --prefix "strings::"    # members of std::strings
lex complete --prefix "@G"           # decorators
lex complete --file main.lex --line 3 --col 11 --json
```

## Language server (`lex lsp`)
Minimal LSP 3.17 over stdio: `initialize`, `textDocument/completion`
(triggers `:`, `@`, `.`), `textDocument/hover`, `textDocument/definition`
(Ctrl+Click → SDK source: real `lib/std/**/*.lex` files for `std::`
packages, `lib/std/native/*.lex` signature stubs for native builtins),
`shutdown`/`exit`. Point any generic LSP-client extension at the command
`lex lsp`.

## REPL
Inside `lex repl`, `:complete <prefix>` lists suggestions
(e.g. `:complete Http::s`), using the same engine.
"#;

/// Scaffold editor assets into `<dir>` (default: current directory).
pub fn init(dir: Option<String>) -> Result<()> {
    let root = match dir {
        Some(d) => PathBuf::from(d),
        None => std::env::current_dir()?,
    };
    let vscode = root.join(".vscode");
    fs::create_dir_all(&vscode)
        .with_context(|| format!("creating {}", vscode.display()))?;
    fs::write(vscode.join("tasks.json"), TASKS_JSON)?;
    fs::write(vscode.join("settings.json"), SETTINGS_JSON)?;
    fs::write(vscode.join("lex.code-snippets"), SNIPPETS_JSON)?;
    fs::write(root.join("LEX-TOOLS.md"), TOOLS_MD)?;
    println!("Editor assets written to {}", root.display());
    println!("  .vscode/tasks.json, .vscode/settings.json, .vscode/lex.code-snippets, LEX-TOOLS.md");
    Ok(())
}

/// Scaffold the zero-build VS Code extension (`editors/vscode-lex` layout)
/// into `<dir>`: grammar + language configuration + snippets + task
/// definitions + problem matchers + readme.
pub fn extension(dir: Option<String>) -> Result<()> {
    let root = match dir {
        Some(d) => PathBuf::from(d),
        None => PathBuf::from("editors").join("vscode-lex"),
    };
    let syntaxes = root.join("syntaxes");
    let snippets = root.join("snippets");
    fs::create_dir_all(&syntaxes)
        .with_context(|| format!("creating {}", syntaxes.display()))?;
    fs::create_dir_all(&snippets)
        .with_context(|| format!("creating {}", snippets.display()))?;
    fs::write(root.join("package.json"), EXT_PACKAGE_JSON)?;
    fs::write(root.join("language-configuration.json"), LANG_CONFIG_JSON)?;
    fs::write(syntaxes.join("lex.tmLanguage.json"), GRAMMAR_JSON)?;
    fs::write(snippets.join("lex.code-snippets"), SNIPPETS_JSON)?;
    fs::write(root.join("README.md"), EXT_README)?;
    println!("VS Code extension written to {}", root.display());
    println!("  package.json, language-configuration.json, syntaxes/lex.tmLanguage.json, snippets/lex.code-snippets, README.md");
    Ok(())
}
