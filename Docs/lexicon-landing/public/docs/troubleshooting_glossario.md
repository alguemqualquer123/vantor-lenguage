# Troubleshooting e Glossário - Lexicon SDK

## 1. Troubleshooting (Resolução de Erros Comuns)

### 1.1 Comando 'lex' não reconhecido
- **Causa**: O binário `lex` não foi adicionado ao seu PATH.
- **Solução**: Reinicie o terminal após a instalação. Se o erro persistir, siga o **Passo 2** do Guia de Instalação Manual.

### 1.2 Erro de Build: 'LEXICON_TARGET' não encontrado
- **Causa**: Variável de ambiente necessária para builds cross-platform ausente.
- **Solução**: Defina o target desejado (ex: `set LEXICON_TARGET=native` no Windows ou `export LEXICON_TARGET=native` no Linux).

### 1.3 IntelliSense não funciona no VS Code
- **Causa**: A extensão Lexicon pode estar em conflito com outras extensões ou não carregou corretamente.
- **Solução**: Reinicie o VS Code. Verifique se a extensão **Lexicon Language Support** está habilitada.

### 1.4 Erro no Deploy: 'Unauthorized'
- **Causa**: Você não está autenticado na Lexicon Cloud.
- **Solução**: Execute `lex login` para vincular sua conta ao SDK.

## 2. Glossário de Termos Técnicos

### 2.1 AST (Abstract Syntax Tree)
Representação em árvore da estrutura do código-fonte usada pelo compilador Lexicon.

### 2.2 Bytecode
Forma intermediária do código compilado que o runtime Lexicon executa.

### 2.3 Cloud-Native
Refere-se a sistemas projetados especificamente para rodar de forma eficiente em ambientes de nuvem e edge computing.

### 2.4 FFI (Foreign Function Interface)
Mecanismo que permite ao Lexicon chamar funções escritas em outras linguagens, como C e Rust.

### 2.5 Hot Reload
Capacidade do ambiente de desenvolvimento de aplicar mudanças no código sem a necessidade de reiniciar o processo manualmente.

### 2.6 Pipe Operator (|>)
Operador usado para passar o resultado de uma expressão como o primeiro argumento de uma função subsequente, facilitando a legibilidade.

### 2.7 WASM (WebAssembly)
Formato de instrução binária para uma máquina virtual baseada em pilha, projetado para execução nativa no navegador.
