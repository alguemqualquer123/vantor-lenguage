# User Manual

Lexicon is a modern programming language focused on native performance, WebAssembly (WASM), and Cloud-Native integration.

## 1. Introduction

Lexicon provides a unique blend of low-level performance (powered by LLVM) and high-level productivity (with decorators and smart pipes).

> **Vision**: The official portal offers downloads, interactive templates, and quick access to the SDK documentation.

## 2. Using the CLI (lex)

Essential commands for your workflow:

```bash
# Create a new project
lex new my-project

# Run with hot reload
lex run main.lex --watch

# Build for WebAssembly
lex build --target wasm
```

## 3. Core Features

- **Native Performance**: Compiled to machine code via LLVM.
- **WASM First**: Native support for WebAssembly runtimes.
- **Cloud Ready**: Deploy your APIs directly to the edge with `lex deploy`.
