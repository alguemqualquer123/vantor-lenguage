# Architecture and APIs

A deep dive into how Lexicon works under the hood.

## 1. Core Architecture

Lexicon is written in **Rust** for maximum performance. It consists of:

- **Lexer/Parser**: Robust tokenization and parsing of `.lex` files.
- **Analysis (TypeChecker)**: Advanced type inference and DI Engine.
- **Codegen**: Geração de código nativo (LLVM) e WebAssembly (WASM).
- **CLI Core**: Fast management of project templates and deployments.

## 2. Injeção de Dependência (DI Engine)

Lexicon includes a native DI container to manage component lifecycle:

```lexicon
@Configuration
class AppConfig {
    @Bean
    fn database_service() -> Database {
        return Database::new("postgres://...");
    }
}
```

## 3. WASM Support

The `lex build --target wasm` command produces optimized `.wasm` modules for:
- Browsers (Next.js, React)
- Edge Functions (Cloudflare Workers)
- Serverless runtimes
