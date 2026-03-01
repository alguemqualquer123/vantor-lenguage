# Documentação Técnica - Lexicon SDK & Portal

## 1. Arquitetura do Sistema
O Lexicon é um ecossistema de baixo nível projetado para alta escalabilidade e baixa latência. Ele é composto por quatro componentes centrais:

### 1.1 Compilador (lexicon-cli)
O `lexicon-cli` (escrito em Rust) é o ponto de entrada principal. Ele gerencia o ciclo de vida do projeto:
- **Lexer/Parser**: Converte código Lexicon em uma Árvore de Sintaxe Abstrata (AST).
- **Backend LLVM/WASM**: Gera binários nativos otimizados ou módulos WebAssembly.
- **Watch System**: Utiliza a crate `notify` para monitorar mudanças no sistema de arquivos.

### 1.2 Language Server (LSP)
O LSP (Language Server Protocol) é implementado como uma extensão para o VS Code:
- **Provider de Autocomplete**: Sugere palavras-chave e tipos.
- **Provider de Definição**: Mapeia identificadores para suas declarações no arquivo atual.
- **Realce de Sintaxe**: Definido em JSON TextMate grammars.

### 1.3 Lexicon Cloud (lex deploy)
A infraestrutura de nuvem da Lexicon é baseada em edge computing:
- **Cloud Runtime**: Um runtime otimizado para executar bytecode Lexicon com isolamento seguro.
- **Edge Deployment**: APIs são implantadas globalmente em menos de 2 segundos.

### 1.4 Native Interop (FFI)
O sistema de interoperação nativa (`lex ffi`) mapeia tipos Lexicon para tipos C/Rust correspondentes:
- **Lexicon String** <-> `*const i8` (C)
- **Lexicon i32** <-> `int32_t` (C)

## 2. APIs de Baixo Nível
### 2.1 core.net.Http
API para criação de servidores e clientes HTTP:
- `Http::serve(addr: String)`: Inicia um servidor no endereço especificado.
- `Http::get(url: String)`: Realiza uma requisição GET assíncrona.

### 2.2 core.json.Json
Biblioteca nativa para serialização e desserialização:
- `Json::parse(str: String)`: Converte uma string JSON em um objeto Lexicon.
- `data.toString()`: Converte um objeto de volta para string JSON.

## 3. Banco de Dados e Fluxo de Dados
### 3.1 Lexicon Cache
O sistema de cache (`lexicon-cache`) utiliza uma estrutura in-memory baseada em hash maps para armazenamento ultra-rápido de dados temporários.

### 3.2 Fluxo de Dados (Pipes)
O Lexicon introduz o operador pipe (`|>`) para fluxos de dados funcionais:
```lexicon
let data = fetch_data()
    |> filter_results()
    |> format_output()
    |> print();
```
O compilador otimiza o fluxo de pipes para evitar alocações desnecessárias na memória heap.
