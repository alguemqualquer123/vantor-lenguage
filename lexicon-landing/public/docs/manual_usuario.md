# Manual do Usuário - Lexicon v0.2.0

> Lexicon **v0.2.0**: supervisor de hot-reload, super extensão VS Code **1.1.x**,
> runner fiel a comentários e `lex check` / `lex vet` funcionando de verdade.

## 1. Introdução

O Lexicon é uma linguagem de programação moderna com toolchain real em Rust:
`lex run`, `lex check`, `lex vet`, `lex test`, `lex lint`, `lex fmt`,
`lex doc` e `lex trace`. Este manual descreve o portal e o fluxo de uso diário.

## 2. Navegação no Portal (Landing Page)

- **Hero**: apresentação da linguagem, badge **v0.2.0** e botões de download + repositório.
- **Recursos**: hot-reload supervisor, super extensão, diagnósticos pré-execução e instalador honesto.
- **Templates + Quickstart**: scaffolds `lex new ...` e comandos reais copiáveis
  (`lex run --watch`, `lex vet`, fluxo demo-api).
- **Downloads**: Windows (`.msi`), Linux (`.tar.gz`), macOS (`.pkg`) e WASM (`.wasm`), todos **v0.2.0**.
- **Documentação**: este manual, instalação, arquitetura e troubleshooting.

## 3. Utilizando o CLI (lex)

```powershell
lex new minha-api --template api --edge   # scaffold de projeto
lex run main.lex                          # compila e executa
lex run --watch main.lex                  # hot reload: salvou → reiniciou
lex check main.lex                        # type checking sem build
lex vet models.lex                        # gate de correção (esperado: "vet: no correctness violations")
lex test                                  # testes @Test
lex lint --json                           # lint legível por máquina
lex fmt                                   # formatação
lex doc routes.lex                        # docs da API em Markdown
lex trace main.lex                        # timings dos estágios
lex install                               # instala globalmente (idempotente)
```

### Sintaxe real (forma `::`)

```lex
import core::net::Http;
import core::json::Json;
import app::models;
import app::store;

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
```

> Imports usam `::` (`core::net::Http`). A forma pontilhada (`core.net.Http`)
> é corrigida por `lex fix`, mas escreva direto com `::`.

## 4. Hot Reload (`lex run --watch`)

O supervisor **nunca executa código do usuário em-processo**: ele gera um filho
`lex run <arquivo>` (sem `--watch`, sem recursão) e o reinicia a cada save.

- **Debounce**: rajadas de save coalescem (300 ms) em **um** restart.
- **Filtro**: só `.lex` dispara; `target/`, `build/`, `.git/`, dotfiles,
  backups (`~`), `.tmp`/`.swp` e `*.db*` são ignorados.
- **Backoff**: se o filho morre em <1 s três vezes seguidas (ex.: porta ocupada),
  o supervisor para em vez de girar em loop.
- **Herança de ambiente**: o filho herda `APP_ENV`, `DATABASE_URL`, etc. —
  dev/prod continua funcionando sob `--watch` (`LEX_SUPERVISED=1` no filho).
- **CI**: com `CI=true`, o watch vira uma execução única supervisionada.

## 5. Super Extensão VS Code 1.1.x

- **85+ snippets** Lexicon-reais (`main`, `fn`, `@Get`, `serve`, `match`, `struct`,
  `@Test`, `Http::get`, `Json::parse`, `print`, bloco `/* */`).
- **4 temas**: Dark Pro, Midnight, Light e High Contrast (+ icon theme).
- **Autocomplete contextual + auto-import**: `Módulo::` lista só membros daquele
  módulo; aceitar a sugestão insere o `import core::x::Y;` que falta.
- **Pre-run syntax gate**: antes de rodar, checagem E0101/E0201 com `linha:col`
  exatas (*Show Problems* / *Run Anyway*); mesmos achados no diagnóstico de save.
- **Vet-on-save**: diagnósticos do `lex vet` direto no editor + problem matcher
  `lexicon-vet` para tasks (`lex run --ci`, `lex run --watch`, `lex test`, `lex check`).
- Sem extensão? `lex ide init` gera `.vscode/tasks.json`, snippets e `LEX-TOOLS.md`.

## 6. Demo API real (HTTP + SQLite + .env)

```powershell
powershell -ExecutionPolicy Bypass -File demo-api\run-dev.ps1    # env=development, sqlite:./dev.db
powershell -ExecutionPolicy Bypass -File demo-api\run-prod.ps1   # env=production,  sqlite:./prod.db
curl http://localhost:3000/users
curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"Bob","email":"bob@example.com"}'
```

- `python3 demo-api/seed.py demo-api/dev.db` cria o banco real (4 usuários).
- `Env::get("NOME")` lê o ambiente **real**: atribua a variável primeiro
  (`let x = Env::get(..)`); inline dentro de `print` sai literal.
- `Http::serve("0.0.0.0:3000")` binda o endereço **literal**; o `.env` troca
  comportamento (env/db/log), não a porta.

## 7. Notas honestas (limites conhecidos)

- `lex run` executa **um arquivo**; módulos irmãos mostram imports/exports e são
  validados por `vet`/`lint`/`fmt`/`doc`.
- Comentários (`//`, `/* */`) são ignorados de verdade: código comentado nunca
  executa, nunca registra rota, nunca sobe servidor.
- `lex deploy` e `lex ffi` hoje são **simulados** (barras de progresso, sem efeito
  real) — não os trate como deploy/binding de produção.
- Suporte a WebView nativo foi removido do binário (não linka no MinGW-GNU);
  nada no fluxo `check`/`build` depende de webview.
