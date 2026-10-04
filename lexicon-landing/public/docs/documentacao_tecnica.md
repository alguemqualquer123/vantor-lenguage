# Documentação Técnica - Lexicon v0.2.0

## 1. Arquitetura do sistema

Crates (workspace Rust): `lexicon-cli`, `lexicon-lexer`, `lexicon-parser`,
`lexicon-analysis`, `lexicon-codegen`, `lexicon-core`, `lexicon-utils`,
`lexicon-async`, `lexicon-http`, `lexicon-cache`, `lexicon-db`, `lexicon-wasm`,
`lexicon-gui` (atrás da feature `gui`).

### 1.1 Pipeline de compilação

1. **Lexer** (`lexicon-lexer`): tokeniza `.lex`, incluindo `::` (PathSep),
   interpolação `{var}`, decorators `@Get/@Post/@Put/@Delete/@Test` e `#[cfg]`.
2. **Parser** (`lexicon-parser`): AST + **recovery com erro visível** — erros
   viram `arquivo:linha:col: msg` e **falham** (sem fallback silencioso).
3. **Análise** (`lexicon-analysis`): type checker, `#[cfg]` por features/target,
   borrow rules (E0305/E0382).
4. **Codegen** (`lexicon-codegen`): backend LLVM IR (`build/output.ll`).

`lex check <arquivo>` roda esse pipeline sem emitir build; `lex build`
aceita `--release`, `--target` e `--features`.

### 1.2 Supervisor de hot reload (`lex run --watch`, Spec §62)

- Supervisor nunca executa código do usuário: gera filho `lex run <arquivo>`
  com `LEX_SUPERVISED=1` e o mata/espera (`kill`+`wait`, kill-on-drop) a cada restart.
- Debounce 300 ms, filtro `.lex` (ignora `target/`, `build/`, `.git/`,
  dotfiles, `~`, `.tmp`/`.swp`, `*.db*`), backoff de 3 mortes <1 s.
- Herda o ambiente (`.env` dev/prod continua valendo sob `--watch`).

### 1.3 Runner fiel a comentários

`strip_comments`/`code_contains` espelham o tokenizer: `print`, `Http::serve`
e rotas comentadas **nunca** executam, registram ou sobem servidor. Runner,
lint e security scan operam sobre código livre de comentários.

## 2. Diagnósticos: `lex check`, `lex vet` e códigos E

`lex vet [arquivo]` = type check + interface + ABI; limpo imprime
`vet: no correctness violations` (security findings são advisory).
Todo diagnóstico carrega código estável (`E0101`, `E0201`, …) para CI/IDEs.

| Código | Significado |
|---|---|
| E0101 / E0102 / E0103 | string não fechada / literal inválido / char inesperado |
| E0201 / E0202 / E0203 | token esperado / atribuição inválida / código inalcançável |
| E0301 / E0302 / E0303 / E0304 | type mismatch / tipo desconhecido / restrição genérica / interface não implementada |
| E0305 / E0382 | violação de borrow / uso após move |
| E0306 | narrowing exige cast `as` |
| E0401 | falha de codegen |
| E0501 | panic de runtime |
| E0601 | erro de IO/fs/compressão |
| E0701 | corrida/concorrência (canal fechado, backpressure, data race) |
| E0801 | unwrap em `None` |

A extensão adiciona o pre-run gate (E0101/E0201 com `linha:col`) e avisos
W0001/W0002, com os mesmos achados no diagnóstico de save.

## 3. APIs de runtime (reais)

- `Http::serve("0.0.0.0:3000")`: sobe o servidor HTTP real (axum, CORS,
  envelopes JSON, 404/405, log) — bloqueia até ser morto, ideal para `--watch`.
  O endereço é **literal**: é ele que binda.
- `Http::get/post`, `Json::parse`, `Env::get`, `Db::connect/query/execute`,
  `Console::writeLine`, `print/println/panic/recover/assert/inspect/spawn`.
- `lex complete --prefix "Http::"` / `--file … --line … --col … [--json]` e
  `lex lsp` (LSP 3.17 via stdio: initialize/completion/hover) usam o mesmo
  registro — o que o autocomplete sugere, o toolchain honra.

## 4. Demo API (demo-api/)

`main.lex` (executável via `lex run`), `models.lex` (`pub struct`/`pub type`/
`pub fn`), `config.lex` (`Env::get`), `store.lex` (`Db::connect/query/execute`
via `lexicon-db`/SQLite), `routes.lex` (handlers `@Get/@Post`), `.env.dev` /
`.env.prod`, `run-dev.ps1` / `run-prod.ps1`, `seed.py` + `dev.db` real.

## 5. Limites honestos da v0.2.0

- `lex deploy` e `lex ffi` são **simulados** (progresso fake, sem efeito real).
- `lex run` executa **um arquivo**; o grafo de módulos é validado, não linkado.
- Sem WebView nativa no binário (crate `webview` removida — não linkava no
  MinGW-GNU); `compiler.rs`/`webview.rs` seguem stubs desvinculados.
- WASM AOT/JIT, LSP completo e package registry seguem no roadmap (itens 66–100).
