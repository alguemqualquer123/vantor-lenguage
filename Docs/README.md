# Lexicon Programming Language (v0.2.0)

Lexicon is a modern programming language with high-level syntax and low-level performance.

> Language version is **v0.2.0** (workspace `Cargo.toml`). The VS Code
> extension is versioned separately (**1.1.x**) — do not confuse the two.

## Running Lexicon

To run Lexicon from the command line, you can use the provided prefix scripts:

### Windows (CMD/PowerShell)
```cmd
.\lex.bat <command> [args]
```

### Unix-like (Bash/Zsh)
```bash
./lex.sh <command> [args]
```

### Common Commands
- `.\lex.bat new <project_name>` - Create a new project.
- `.\lex.bat run <file.lex>` - Run a Lexicon file.
- `.\lex.bat run --watch <file.lex>` - Run with hot reload (save a `.lex` file to auto-restart; 300 ms debounce, `.lex`-only filter, crash backoff).
- `.\lex.bat check <file.lex>` - Type-check without building.
- `.\lex.bat vet <file.lex>` - Static correctness checks (+ advisory security findings).
- `.\lex.bat lint` / `fmt` / `doc` / `trace` - Lint, format, API docs, execution trace (all comment-accurate).
- `.\lex.bat visualize <file.lex>` - Visualize data flow in pipes.
- `.\lex.bat repl` - Start the interactive shell.
- `.\lex.bat build` - Build the project.
- `.\lex.bat version` - Show versions.
