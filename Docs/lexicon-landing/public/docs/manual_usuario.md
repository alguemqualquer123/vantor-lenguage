# Manual do Usuário - Lexicon SDK & Portal

## 1. Introdução
O Lexicon é uma linguagem de programação moderna focada em performance nativa, WebAssembly (WASM) e integração nativa com nuvem (Cloud-Native). Este manual descreve como navegar no portal oficial e utilizar as ferramentas do SDK.

## 2. Navegação no Portal (Landing Page)
O portal oficial (`http://localhost:3002`) é dividido em seções principais:
- **Hero Section**: Apresentação da linguagem e botão principal de download do instalador `.msi` (Windows).
- **Recursos (Features)**: Descrição detalhada das capacidades técnicas (Performance, Cloud, Type Safety, etc.).
- **Templates de Projeto**: Exemplos interativos de como iniciar projetos (API REST, WASM Plugin, Microservices).
- **Versionamento/Downloads**: Área para baixar versões específicas para Windows, Linux, macOS e WASM.
- **Documentação**: Link direto para este manual técnico e guias de linguagem.

## 3. Utilizando o CLI (lex)
Após a instalação, o comando `lex` estará disponível no seu terminal.

### Comandos Principais:
- `lex new <nome>`: Cria um novo projeto Lexicon.
- `lex run <arquivo>`: Compila e executa um arquivo `.lex`.
- `lex run --watch`: Ativa o **Hot Reload** (recompila automaticamente ao salvar).
- `lex build --target wasm`: Compila o projeto para WebAssembly.
- `lex test`: Executa testes unitários (funções marcadas com `@Test`).
- `lex deploy`: Realiza o deploy instantâneo para a Lexicon Cloud.
- `lex ffi <lib>`: Gera bindings para bibliotecas C/Rust.

## 4. Funcionalidades de Desenvolvimento
### 4.1 Hot Reload
Ao utilizar o comando `lex run --watch`, o compilador monitora mudanças nos arquivos. Quando um arquivo `.lex` é salvo, o processo é reiniciado instantaneamente, permitindo um ciclo de desenvolvimento ultra-rápido.

### 4.2 IntelliSense (VS Code)
A extensão oficial para VS Code oferece:
- **Autocomplete**: Sugestão de palavras-chave e tipos.
- **Go to Definition**: Navegação rápida para definições (F12).
- **Hover**: Informações de tipo ao passar o mouse.

### 4.3 Integrated Testing
Para testar seu código, decore suas funções com `@Test`:
```lexicon
@Test
fn test_soma() {
    let resultado = soma(2, 2);
    assert resultado == 4;
}
```
Execute `lex test` para ver o relatório de testes no terminal.
