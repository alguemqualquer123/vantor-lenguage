# lex-api — API HTTP de demonstração em Lexicon

API de usuários em Lexicon executada pelo binário já compilado
(`../target/debug/lex.exe`), cobrindo: **imports/exports**,
**funções e variáveis**, **SQLite**, **variáveis de ambiente
dev vs prod** e o **toolchain** (`vet`, `lint`, `fmt`, `doc`, `trace`).

> Tudo abaixo foi executado de verdade contra o binário — saídas reais
> estão nas seções. Limitações conhecidas do binário estão no final,
> sem maquiagem.

## Estrutura

```text
demo-api/
├── main.lex      # API executável (`lex run main.lex`) — usa só print/println
├── models.lex    # struct User, type UserId, fns (exporta com `pub`)
├── config.lex    # lê APP_ENV / DATABASE_URL / LOG_LEVEL via Env::get
├── store.lex     # acesso SQLite em Lexicon (Db::connect/query/execute)
├── routes.lex    # handlers @Get/@Post (importa models + store)
├── .env.dev      # APP_ENV=development, sqlite:./dev.db, LOG_LEVEL=debug
├── .env.prod     # APP_ENV=production,  sqlite:./prod.db, LOG_LEVEL=warn
├── run-dev.ps1   # carrega .env.dev e sobe a API
├── run-prod.ps1  # carrega .env.prod e sobe a API
├── seed.py       # cria o banco SQLite REAL (só stdlib do Python)
├── dev.db        # banco gerado (4 usuários, espelha GET /users)
└── README.md     # este arquivo
```

## 1. Rodando (dev vs prod)

```powershell
# desenvolvimento
powershell -ExecutionPolicy Bypass -File demo-api\run-dev.ps1
# produção
powershell -ExecutionPolicy Bypass -File demo-api\run-prod.ps1
```

> Dica: rode de dentro de `demo-api/` (`.\run-dev.ps1`). Se o caminho
> tiver espaços/colchetes e o `-File` reclamar, use
> `powershell -ExecutionPolicy Bypass -Command "& '.\run-dev.ps1'"`.

Banner real impresso com `print`/`println` (valores vêm do ambiente):

```text
# dev
=== lex-api ===
env=development
db=sqlite:./dev.db
log=debug
routes: / /hello /users /stats /health /config POST /users
[Server running on http://localhost:3000]

# prod
=== lex-api ===
env=production
db=sqlite:./prod.db
log=warn
```

Como funciona (`config.lex` + `main.lex`): `Env::get("NOME")` lê o
ambiente **real** do processo. Padrão que funciona no binário atual:

```lex
let app_env = Env::get("APP_ENV");   // atribua a variável primeiro
print("env=" + app_env);             // ...e use a variável no print
```

## 2. Imports, funções, variáveis, exports

```lex
// main.lex — importa módulos core e os arquivos irmãos:
import core::net::Http;
import core::json::Json;
import app::config;
import app::models;
import app::store;
import app::routes;
```

```lex
// models.lex — `pub` exporta; sem `pub` é privado do arquivo:
pub struct User {
    id: int;
    name: String;
    email: String;
}
pub type UserId = int;

pub fn user_label(name: String) -> String {
    return "user:" + name;
}

fn normalize_email(email: String) -> String {  // privada
    return email;
}
```

```lex
// routes.lex — importa e USA outro módulo (funções e valores):
import app::models;
import app::store;

@Get("/users")
pub fn users() -> String {
    return store::all_users();   // chamada qualificada: módulo::função
}
```

Variáveis: `let` (imutável) e `let mut`/`var` (mutável), com ou sem tipo:

```lex
let app_env = Env::get("APP_ENV");
let port: int = 3000;
```

Validação real pelo toolchain (todas passam nesta demo):

```powershell
.\target\debug\lex.exe vet demo-api\models.lex   # vet: no correctness violations
.\target\debug\lex.exe vet demo-api\store.lex
.\target\debug\lex.exe doc demo-api\routes.lex   # gera Markdown da API
.\target\debug\lex.exe trace demo-api\main.lex   # lex: 254 tokens, parse: 18 decls
```

## 3. Endpoints (testados com curl)

| Método | Rota     | Handler      | Resposta real |
|--------|----------|--------------|---------------|
| GET    | `/`      | `root`       | `{"data":"Welcome to Lexicon API!",...}` |
| GET    | `/hello` | `hello`      | `{"data":"Hello from Lexicon HTTP Server!",...}` |
| GET    | `/users` | `users`      | lista John/Jane/Bob/Alice |
| GET    | `/stats` | `stats`      | `{"data":0,"message":"Total requests",...}` |
| GET    | `/health`| `health`     | eco `{endpoint, method}` |
| GET    | `/config`| `config`     | eco `{endpoint, method}` |
| POST   | `/users` | `create_user`| `{"message":"User created successfully!",...}` |

```powershell
curl http://localhost:3000/users
curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"Bob","email":"bob@example.com"}'
```

### Moldando respostas: status, headers, cookies, query

Decorators acima do handler (um por linha, valem só para aquela rota):

```lex
@Status(201)
@Header("X-App: lex-api")
@Cookie("session=abc123; Path=/; HttpOnly")
@Post("/items")
pub fn create_item() -> String {
    return "made";   // vira o "data" da resposta
}
```

- `@Status(N)` troca o status (100–599; fora disso é ignorado).
- `@Header("Nome: valor")` insere response headers (nomes inválidos
  são ignorados, nunca derrubam o servidor).
- `@Cookie("...")` emite um `Set-Cookie` por linha.
- O `return "..."` do handler vira o `"data"` (rotas genéricas).
- Query strings e cookies de requisição são ecoados (`"query"`,
  `"cookies"`) nas respostas genéricas; corpos POST aceitam JSON
  (limite 1 MiB, 400 em JSON inválido, 413 se estourar).
- Toda resposta carrega `X-Powered-By: lexicon/0.2.0`,
  `X-Request-Id: lex-N` e CORS liberado (`OPTIONS` → 204);
  rota inexistente → 404 JSON, método errado → 405 JSON.
- Rotas `@Put`/`@Delete` funcionam como `@Get`/`@Post`.

## 4. SQLite

`store.lex` mostra o uso idiomático (mesmo estilo `Módulo::função` do `Http`/`Json`):

```lex
import app::config;

pub fn connect() -> String {
    let url = config::db_url();
    return Db::connect(url);
}

pub fn migrate() -> String {
    return Db::execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)");
}

pub fn all_users() -> String {
    return Db::query("SELECT id, name, email FROM users");
}
```

Banco **real**: `python3 demo-api/seed.py demo-api/dev.db` cria `dev.db`
com a tabela `users` e os mesmos 4 registros do `GET /users` —
inspecionável com qualquer cliente SQLite. A execução do SQL acontece
via o runtime `lexicon-db` (SQLite/sqlx); `lex run` valida o código
(`vet` limpo acima) e serve o HTTP.

## 5. Hot reload (`lex run --watch`)

```powershell
# dentro de demo-api/ (com o env já exportado pelo run-dev.ps1),
# ou adaptando o run-dev.ps1 para chamar `lex run --watch`:
..\target\debug\lex.exe run --watch demo-api\main.lex
```

Comportamento real do supervisor (`src/lexicon-cli/src/compiler.rs`, `watch()`):

- Salve um `.lex` → o supervisor **mata o filho e relança
  `lex run main.lex` sem `--watch`** (sem recursão; `LEX_SUPERVISED=1`
  no filho pula o `Press Enter to exit`).
- **Debounce**: rajadas de save são coalescidas (300 ms) em **um** restart.
- **Filtro**: só `.lex` dispara; `target/`, `build/`, `.git/`,
  dotfiles, backups (`~`), `.tmp`/`.swp` e `*.db*` são ignorados —
  o `dev.db` pode mudar sem reiniciar o servidor.
- **Backoff**: se o filho morrer em < 1 s três vezes seguidas
  (ex.: porta ocupada), o supervisor para em vez de girar em loop.
- O filho herda o ambiente do supervisor, então `APP_ENV` /
  `DATABASE_URL` / `LOG_LEVEL` continuam valendo sob `--watch`.

## 6. Limitações conhecidas (binário atual)

- `lex run` executa **um arquivo**; os módulos irmãos mostram o sistema
  de imports/exports e são validados por `vet`/`lint`/`fmt`/`doc`.
- Comentários (`//`, `/* */`) são **ignorados de verdade** via
  `strip_comments` (`src/lexicon-lexer/src/comments.rs`): código
  comentado nunca executa, nunca registra rota e nunca sobe servidor —
  rotas/portas são extraídas do código **sem comentários**, com números
  de linha preservados. (Em binários anteriores a 2026-09-29, comentado
  executava.)
- `Env::get` avalia quando atribuído a variável (`let x = Env::get(..)`);
  inline dentro do `print` sai literal (`eval_expr` não tem ramo para
  chamada `Env::get`, só o caminho de atribuição resolve o ambiente).
- `Http::serve("0.0.0.0:3000")` usa endereço **literal** (`extract_port`
  faz parse do literal; é ele que o servidor binda); o `.env` troca
  comportamento (env/db/log), não a porta.
- `lex check` e `lex vet` funcionam (typecheck real; `vet` adiciona
  avisos de segurança como advisory). `lint`/`fmt`/`doc`/`trace`
  operam sobre código sem comentários.
- `Db::connect/query/execute` são **validados** pelo toolchain
  (`vet` limpo acima) e o runtime `lexicon-db` (sqlx, SQLite) existe no
  workspace; mas o runner não interpreta chamadas `Db::` por request —
  as respostas HTTP vêm dos handlers axum (dados espelhando `dev.db`).
  `GET /users` fora da rota registrada devolve 404 JSON
  (`{"success": false, "data": null, "message": "no route for ..."}`);
  método errado na rota existente devolve 405 JSON.
- Rode sempre com env carregado (`run-dev.ps1`/`run-prod.ps1`); sem isso
  `Env::get` devolve `NOT_FOUND` — é o comportamento correto, não erro.
