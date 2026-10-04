# Guia de Instalação e Configuração - Lexicon v0.2.0

## 1. Pré-requisitos

- SO: Windows 10+, Linux (Ubuntu 20.04+), macOS (Catalina+).
- Disco: 500 MB livres.
- Internet: só para download do SDK (o toolchain roda 100% local).

## 2. Instalação automática silenciosa (recomendado)

Todo `lex` executa um **auto-install silencioso**: copia o binário para
`~/.lexicon/bin` (só se mudou) e adiciona ao PATH **apenas se ausente**
(checa o processo E o registro persistente — sem duplicatas, sem output).

```powershell
# Windows (PowerShell)
iwr https://get.lexicon.dev/install.ps1 -useb | iex
```

```bash
# Linux e macOS
curl -fsSL https://get.lexicon.dev | sh
```

Para pular o auto-install (CI/benchmarks):

```bash
CI=true lex run main.lex
# ou
LEXICON_NO_AUTO_INSTALL=1 lex run main.lex
```

## 3. Instalação manual

```bash
lex install      # copia o binário e registra no PATH (idempotente)
lex uninstall    # remove o binário e limpa ~/.lexicon vazio
```

- Se `~/.lexicon/bin` já está no PATH, o `lex install` imprime
  "already in your system PATH" e não duplica nada.
- PATH no Windows é gravado no registro do usuário (`HKCU\Environment`);
  **reabra o terminal** para a sessão nova enxergar a entrada.

## 4. Configuração do VS Code

Opção A — super extensão (recomendado, v1.1.x):

1. Abra o VS Code → **Extensões** (Ctrl+Shift+X).
2. Procure **Lexicon Super** (`lexicon-team.lexicon-super`).
3. Instale, recarregue e abra qualquer `.lex` — gramática, 85+ snippets,
   4 temas, autocomplete contextual, pre-run gate e vet-on-save funcionam juntos.

Opção B — sem extensão (scaffold do toolchain):

```bash
lex ide init     # gera .vscode/tasks.json + snippets + LEX-TOOLS.md
```

Isso cria as tasks `lex run --ci`, `lex run --watch` (hot reload), `lex test`
e `lex check` no arquivo atual, além de instruções para apontar qualquer
cliente LSP genérico para `lex lsp`.

## 5. Verificação da instalação

```bash
lex --version
```

O retorno esperado na **v0.2.0** identifica o binário da linguagem. Em seguida,
valide o toolchain de verdade:

```bash
lex vet demo-api\models.lex   # esperado: "vet: no correctness violations"
lex check demo-api\main.lex   # esperado: "Type checking passed!"
```

Se `lex` não for reconhecido: reabra o terminal (PATH persistido exige sessão
nova) e rode `lex install` de novo — ele é idempotente e diz exatamente
o que faltava.
