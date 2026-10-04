# Troubleshooting e Glossário - Lexicon v0.2.0

## 1. Troubleshooting (erros comuns e reais)

### 1.1 Comando `lex` não reconhecido

- **Causa**: `~/.lexicon/bin` ainda não está no PATH da sessão.
- **Solução**: reabra o terminal e rode `lex install` (idempotente — nunca
  duplica). No Windows o PATH persiste no registro do usuário e exige sessão nova.

### 1.2 `file.lex:line:col: msg` antes de rodar

- **Causa**: o pre-run gate (ou `lex check`/`lex vet`) encontrou erro de sintaxe/tipo.
- **Solução**: não é bug do runner — corrija a `linha:col` indicada. Erros comuns:
  E0101 (string não fechada), E0201 (token esperado, ex. `;`), E0301 (type mismatch).
  Dica: `lex fix --dry-run` mostra migrações seguras (ex. `Http.get(` → `Http::get(`).

### 1.3 Hot reload parou: "child keeps dying fast"

- **Causa**: o filho morreu em <1 s três vezes (porta ocupada é o clássico).
- **Solução**: libere a porta do `Http::serve("0.0.0.0:3000")` literal ou corrija
  o erro, e reinicie o `lex run --watch`. Backoff é proteção, não bug.

### 1.4 `Env::get` devolve `NOT_FOUND`

- **Causa**: processo sem env carregado — comportamento correto, não erro.
- **Solução**: rode com `demo-api\run-dev.ps1` / `run-prod.ps1`; e atribua a
  variável primeiro (`let x = Env::get(..)`), pois inline em `print` sai literal.

### 1.5 Código comentado "não executa" / rota sumiu

- **Causa**: nenhuma — é o correto na v0.2.0. Comentários são removidos antes de
  runner/lint/scan; `print` ou `Http::serve` comentado nunca executa nem registra rota.

### 1.6 IntelliSense/vet não aparece no VS Code

- **Causa**: extensão antiga ou `lex` fora do PATH (`lexicon.path`).
- **Solução**: atualize a **Lexicon Super** para 1.1.x, rode `lex install` e
  confira `lex vet <arquivo>` no terminal — o vet-on-save espelha esse resultado.

### 1.7 `lex deploy` "funcionou" mas nada publicou / `lex ffi` sem bindings

- **Causa**: ambos são **simulados** na v0.2.0 (output cosmético).
- **Solução**: não use em produção; acompanhe o roadmap para o status real.

## 2. Glossário

- **AST**: árvore de sintaxe gerada por lexer/parser a partir de `.lex`.
- **Backoff (hot reload)**: parada após 3 mortes rápidas do filho (<1 s).
- **Comment-accurate runner**: runner opera sobre código sem comentários.
- **Debounce (300 ms)**: coalescência de saves em um único restart.
- **E-codes (E0101–E0801)**: códigos estáveis de diagnóstico para CI/IDEs.
- **FFI**: interface para C/Rust (`lex ffi` hoje simulado).
- **Hot Reload**: `lex run --watch` — save → auto-restart supervisionado.
- **LEX_SUPERVISED=1**: marcador de ambiente do filho supervisionado.
- **Pipe (`|>`)**: encadeamento funcional de dados.
- **Pre-run gate**: barreira de sintaxe da extensão antes de executar.
- **Vet (`lex vet`)**: checagens estáticas de correção (tipo/interface/ABI).
- **Vet-on-save**: diagnósticos do vet dentro do editor ao salvar.
- **WASM**: alvo WebAssembly (`--target wasm` no scaffold).
