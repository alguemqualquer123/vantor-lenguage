# Manual do Usuário - Lexicon v0.3.5

> Lexicon **v0.3.5**: toolchain de um binário só, supervisor de hot-reload,
> motor gráfico em wgpu, `lex mod`, 89 pacotes de stdlib, super extensão VS Code
> **1.2.0** e `lex check` / `lex vet` funcionando de verdade.

## 1. Introdução

O Lexicon é uma linguagem de programação moderna com toolchain real em Rust:
`lex run`, `lex check`, `lex vet`, `lex test`, `lex lint`, `lex fmt`,
`lex doc`, `lex trace` e `lex mod`. Este manual descreve o portal e o fluxo de
uso diário.

## 2. Navegação no Portal (Landing Page)

- **Hero**: apresentação da linguagem, badge **v0.3.5**, dois botões de download
  (SDK completo / só a linguagem) e repositório. O trecho de código exibido roda
  como está: `lex run hero.lex`.
- **Recursos**: hot-reload supervisor, super extensão, diagnósticos pré-execução
  e motor gráfico wgpu.
- **Download no GitHub**: os dois sabores, com tamanho real dos arquivos,
  comandos de instalação por script, plataformas, checksums SHA-256 e link para
  todas as releases. Tudo sai dos assets da release
  [`alguemqualquer123/vantor-lenguage`](https://github.com/alguemqualquer123/vantor-lenguage/releases).
- **Templates + Quickstart**: scaffolds `lex new ...` (api, gui, plugin, service)
  e comandos reais copiáveis (`lex run --watch`, `lex vet`, fluxo demo-api).
- **Documentação**: este manual, instalação, arquitetura e troubleshooting.

## 3. Utilizando o CLI (lex)

```powershell
lex new minha-api --template api --edge   # scaffold de projeto
lex new minha-janela -t gui               # janela nativa (wgpu) com loop + input
lex run main.lex                          # compila e executa
lex run --watch main.lex                  # hot reload: salvou → reiniciou
lex run --ci main.lex                     # execução única supervisionada (fecha a janela)
lex check main.lex                        # type checking sem build
lex vet models.lex                        # gate de correção (esperado: "vet: no correctness violations")
lex test                                  # testes @Test
lex lint --json                           # lint legível por máquina
lex fmt                                   # formatação
lex doc routes.lex                        # docs da API em Markdown
lex trace main.lex                        # timings dos estágios
lex mod add github:alice/texto-utils      # dependência fixada em lexicon.lock
lex sdk verify                            # confere o SDK instalado (116 arquivos)
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

## 5. Super Extensão VS Code 1.2.0

A extensão é `lexicon-team.lexicon-super` (v1.2.0) e vem como asset da release
(`lexicon-super-1.2.0.vsix`). Instale com `code --install-extension
lexicon-super-1.2.0.vsix`.

- **117 snippets** Lexicon-reais (`main`, `fn`, `@Get`, `serve`, `match`, `struct`,
  `@Test`, `Http::get`, `Json::parse`, `print`, bloco `/* */`).
- **4 temas** (Dark Pro, Midnight, Light, High Contrast) + icon theme.
- **Autocomplete contextual + auto-import**: `Módulo::` lista só membros daquele
  módulo; aceitar a sugestão insere o `import core::x::Y;` que falta.
- **Pre-run syntax gate**: antes de rodar, checagem E0101/E0201 com `linha:col`
  exatas (*Show Problems* / *Run Anyway*); mesmos achados no diagnóstico de save.
- **Vet-on-save**: diagnósticos do `lex vet` direto no editor + problem matcher
  `lexicon-vet` para tasks (`lex run --ci`, `lex run --watch`, `lex test`, `lex check`).
- **Comandos da paleta**: `lexicon.run`, `check`, `vet`, `lint`, `fmt`, `doc`,
  `trace`, `test`, `build`, `debug`, `newProject` — todos chamando o `lex` real
  (localize o binário em `lexicon.path`).
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
- O motor gráfico é **real** (wgpu por trás de `Window`/`Canvas`/`Input`): o
  backend é escolhido pela placa (Vulkan, DX12, Metal, OpenGL, WebGPU). Em CI,
  `lex run --ci` fecha a janela sozinho depois de ~1 s para não travar pipeline.
- Suporte a WebView nativo foi removido do binário (não linka no MinGW-GNU);
  nada no fluxo `check`/`build` depende de webview.
