## Plano da Super Linguagem – Compilador Rápido com LLVM

- **Objetivo principal**: criar uma linguagem de programação moderna, com compilador muito rápido, binários eficientes (via LLVM), sistema de log robusto e runtime com `Console::` e outras APIs essenciais.
- **Tecnologias sugeridas**: C++ + LLVM (backend), CMake (build), testes unitários (GoogleTest ou similar).

---

## Fase 0 – Metas e requisitos

- [ ] **Definir objetivos da linguagem**
  - [ ] Especificar domínio principal (scripts, sistemas, games, webassembly, etc.).
  - [ ] Definir paradigma(s) principais (imperativa, OO, funcional, híbrida).
  - [ ] Definir prioridades: tempo de compilação, performance de execução, ergonomia de sintaxe, segurança (tipos, memória).
- [ ] **Definir plataformas alvo**
  - [ ] Sistemas operacionais (Windows, Linux, macOS).
  - [ ] Arquiteturas (x86_64, ARM64, etc.).
  - [ ] Suporte futuro a WebAssembly (opcional).
- [ ] **Escolher licença e modelo de desenvolvimento**
  - [ ] Licença open-source (por exemplo, MIT, Apache-2.0).
  - [ ] Fluxo de contribuição (issues, PRs, code review).

---

## Fase 1 – Especificação da linguagem

- [ ] **Sintaxe básica**
  - [ ] Definir como é um programa mínimo (função `main`, imports/módulos).
  - [ ] Definir declaração de variáveis (tipadas, inferência de tipo se existir).
  - [ ] Definir declaração de funções (parâmetros, retorno, visibilidade).
  - [ ] Definir blocos, escopos e regras de indentação/chaves.
  - [ ] Definir comentários (`//`, `/* ... */` ou outro estilo).
- [ ] **Controle de fluxo**
  - [ ] `if`, `else if`, `else`.
  - [ ] `while`, `for`, `break`, `continue`.
  - [ ] `return` em funções.
  - [ ] Planejar exceções ou outro mecanismo de erro (se existir).
- [ ] **Sistema de tipos**
  - [ ] Definir tipos primitivos (`int`, `float`, `bool`, `string`, etc.).
  - [ ] Definir tipos compostos (arrays, slices, structs, enums).
  - [ ] Definir regras de conversão implícita e explícita (casts).
  - [ ] Definir se haverá generics, traits/interfaces (se sim, em qual fase).
- [ ] **Runtime conceitual**
  - [ ] Definir o namespace `Console` (por exemplo, `Console::log`, `Console::error`, `Console::readLine`).
  - [ ] Definir APIs padrão mínimas (tempo, arquivos, sistema operacional – até onde o runtime atinge).
  - [ ] Definir estratégia de memória (GC, ARC, manual, híbrido).
- [ ] **Gramática formal**
  - [ ] Escrever gramática EBNF para a linguagem (expressões, statements, declarações).
  - [ ] Anotar ambiguidades e decisões de precedência/associatividade.

---

## Fase 2 – Arquitetura do compilador

- [ ] **Desenhar arquitetura em camadas**
  - [ ] Definir módulos principais: `Lexer`, `Parser`, `AST`, `Semantic`, `IRGen`, `Codegen`, `Runtime`.
  - [ ] Definir módulo de infraestrutura: `Logger`, `Diagnostics`, `CLI`, `Config`.
  - [ ] Desenhar diagrama de dependências (quem importa quem).
- [ ] **Estrutura de pastas do projeto**
  - [ ] Criar diretórios para fonte (`src/`), include (`include/`), testes (`tests/`), runtime (`runtime/`), exemplos (`examples/`), docs (`docs/`).
  - [ ] Configurar CMake ou build system equivalente.
- [ ] **Definir interfaces principais**
  - [ ] Interface do `Lexer` (ex: `Token nextToken()`).
  - [ ] Interface do `Parser` (ex: `std::unique_ptr<Program> parse()`).
  - [ ] Interface de logging (ex: `Logger::info`, `Logger::debug`, `Logger::error`).
  - [ ] Interface de diagnóstico (erros/avisos com localização).
  - [ ] Interface de geração de IR (visitor na AST).

---

## Fase 3 – Lexer (Tokenização perfeita e rápida)

- [ ] **Definir tipos de token**
  - [ ] Palavras-chave (if, else, while, return, etc.).
  - [ ] Identificadores.
  - [ ] Literais numéricos (inteiros, floats).
  - [ ] Literais de string e caractere.
  - [ ] Operadores (aritméticos, lógicos, comparação, atribuição).
  - [ ] Pontuação (parênteses, chaves, colchetes, vírgulas, ponto e vírgula, etc.).
  - [ ] Comentários e espaços em branco.
  - [ ] Token EOF.
- [ ] **Implementar o lexer**
  - [ ] Implementar leitura com buffer eficiente (evitar cópias desnecessárias).
  - [ ] Implementar reconhecimento de identificadores e palavras-chave (tabela de palavras-chave).
  - [ ] Implementar parsing de literais numéricos (incluindo bases diferentes se desejado).
  - [ ] Implementar parsing de strings com caracteres de escape.
  - [ ] Implementar ignorância de espaços e comentários.
  - [ ] Rastrear posição (linha, coluna, índice de arquivo) por token.
- [ ] **Erros de tokenização**
  - [ ] Definir mensagens para caracteres inesperados.
  - [ ] Definir comportamento para literais malformados.
  - [ ] Integrar com sistema de diagnósticos (arquivo/linha/coluna).
- [ ] **Testes de lexer**
  - [ ] Criar suíte de testes unitários para tokens básicos.
  - [ ] Adicionar testes com casos limite (strings grandes, unicode se suportado).
  - [ ] Adicionar testes de fuzzing simples (entradas aleatórias → sem crash).

---

## Fase 4 – Parser e AST

- [ ] **Projeto da AST**
  - [ ] Definir tipos de nós: `Program`, `FunctionDecl`, `VarDecl`, `Block`, `If`, `While`, `For`, `Return`, `Call`, `BinaryExpr`, `UnaryExpr`, `Literal`, `Identifier`, etc.
  - [ ] Definir representação de tipos dentro da AST (anotações de tipo).
  - [ ] Definir nós para módulos/packages (se houver).
- [ ] **Escolher e implementar estratégia de parsing**
  - [ ] Escolher parser recursivo descendente (ou outra estratégia adequada).
  - [ ] Implementar parsing de expressões com precedência (ex: Pratt parser ou tabela de precedência).
  - [ ] Implementar parsing de statements (if, while, for, return, etc.).
  - [ ] Implementar parsing de declarações de funções e variáveis.
- [ ] **Tratamento de erros de sintaxe**
  - [ ] Implementar mensagens de erro claras com contexto (trecho de código).
  - [ ] Implementar recuperação de erro (sincronização em `;` ou `}`).
  - [ ] Integrar com sistema de diagnósticos.
- [ ] **Testes de parser**
  - [ ] Testar programas válidos (comparando com AST esperada).
  - [ ] Testar programas inválidos (garantir erros adequados, sem crash).
  - [ ] Testar arquivos maiores para garantir performance aceitável.

---

## Fase 5 – Análise semântica

- [ ] **Tabelas de símbolos e escopos**
  - [ ] Implementar estrutura de escopos aninhados (global, função, bloco).
  - [ ] Registrar variáveis, funções, tipos e namespaces (`Console`, por exemplo).
  - [ ] Suportar shadowing de identificadores se a linguagem permitir.
- [ ] **Checagem de tipos**
  - [ ] Implementar regras de tipo para operações aritméticas e lógicas.
  - [ ] Implementar checagem de chamadas de função (número e tipos de argumentos).
  - [ ] Implementar checagem de retorno em funções (tipo correto, caminho de código).
  - [ ] Implementar coerções implícitas permitidas e proibir as proibidas.
- [ ] **Outras verificações semânticas**
  - [ ] Variáveis usadas antes de serem declaradas.
  - [ ] Variáveis declaradas e nunca usadas (warnings).
  - [ ] Funções declaradas e nunca usadas (opcional).
  - [ ] Verificação de alcance/vida útil de variáveis (se for relevante).
- [ ] **Testes de semântica**
  - [ ] Programas com erros de tipo esperados (devem gerar diagnósticos corretos).
  - [ ] Programas válidos que exercitam várias combinações de tipos.
  - [ ] Casos de borda (recursão, muitos escopos aninhados).

---

## Fase 6 – Integração com LLVM (IR e geração de código)

- [ ] **Configuração do LLVM**
  - [ ] Adicionar dependência do LLVM ao projeto (CMake, paths, versões suportadas).
  - [ ] Criar módulo de integração (por exemplo, `IRGenerator`).
  - [ ] Inicializar `LLVMContext`, `Module`, `IRBuilder`.
- [ ] **Mapeamento de tipos**
  - [ ] Mapear tipos primitivos da linguagem para tipos LLVM (`i32`, `i64`, `double`, etc.).
  - [ ] Mapear arrays/structs para tipos compostos LLVM.
  - [ ] Definir convenções de chamada (calling convention) para funções.
- [ ] **Geração de IR a partir da AST**
  - [ ] Implementar visitor da AST que gera IR para:
    - [ ] Expressões literais.
    - [ ] Expressões binárias e unárias.
    - [ ] Chamadas de função.
    - [ ] Declarações de variável (alloca, store, load).
    - [ ] Controle de fluxo (`if`, `while`, `for`) com blocos básicos e `br`/`condbr`.
  - [ ] Integrar com tabela de símbolos para resolver referências de funções e variáveis.
- [ ] **Otimizações com LLVM**
  - [ ] Configurar `PassManager` com passes básicos em modo `-O0`.
  - [ ] Adicionar modos `-O1`, `-O2`, `-O3` ajustando o conjunto de passes.
  - [ ] Medir impacto de cada nível de otimização no tempo de compilação e na performance do binário.
- [ ] **Geração de artefatos finais**
  - [ ] Emitir LLVM IR textual (`.ll`) para debug.
  - [ ] Emitir arquivo objeto (`.o`/`.obj`).
  - [ ] Integrar com linker do sistema para gerar executável.
  - [ ] Planejar geração de bibliotecas estáticas/dinâmicas (futuro).

---

## Fase 7 – Runtime e `Console::`

- [ ] **Projeto do runtime**
  - [ ] Definir APIs de runtime expostas à linguagem (I/O, tempo, sistema).
  - [ ] Definir módulo `Console` com funções principais:
    - [ ] `Console::log(...)`
    - [ ] `Console::error(...)`
    - [ ] `Console::write(...)`
    - [ ] `Console::writeln(...)`
    - [ ] `Console::readLine()`
  - [ ] Definir como o runtime será linkado (estático/dinâmico).
- [ ] **Implementação em C++**
  - [ ] Criar biblioteca de runtime em C++.
  - [ ] Implementar funções compatíveis com chamadas geradas pelo LLVM (C ABI).
  - [ ] Implementar conversão de tipos da linguagem para tipos C/LLVM e vice-versa (ex: strings).
- [ ] **Integração com o compilador**
  - [ ] Declarar funções externas no IR para chamadas de runtime.
  - [ ] Garantir que o linker encontre o runtime (flags corretas na CLI).
- [ ] **Testes de runtime**
  - [ ] Programas de exemplo com uso intenso de `Console::`.
  - [ ] Testes de performance (muitas chamadas de log, I/O em loops).

---

## Fase 8 – Sistema de log (compilador + runtime)

- [ ] **Design do logger**
  - [ ] Definir níveis de log: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`.
  - [ ] Definir formato padrão de linha de log (timestamp, nível, módulo, mensagem).
  - [ ] Definir destinos de log: console, arquivo, ambos.
  - [ ] Definir política de rotação de logs (opcional, para arquivos grandes).
- [ ] **Logger do compilador**
  - [x] Implementar logger baseado em `log` + `env_logger`, inicializado no `main` da CLI.
  - [x] Integrar logs nas fases: lexer, parser, semântica, IR, codegen (função `compile` do CLI já emite `trace`/`debug`/`info`/`error`).
  - [ ] Adicionar flags na CLI:
    - [ ] `--log-level=<level>`
    - [ ] `--log-file=<caminho>`
    - [ ] `--debug-ast`
    - [ ] `--debug-ir`
- [ ] **Logger do runtime**
  - [ ] Implementar logs para chamadas de `Console::` e outras APIs de runtime.
  - [ ] Permitir configuração via variáveis de ambiente ou flags de compilação (`-DDEBUG`, por exemplo).
- [ ] **Testes de logging**
  - [ ] Verificar se diferentes níveis de log funcionam corretamente.
  - [ ] Verificar se logs em arquivo respeitam path e permissões.
  - [ ] Testar impacto de logging pesado na performance (e permitir desativar totalmente).

---

## Fase 9 – Performance e “compilação perfeita e rápida”

- [ ] **Medição e profiling**
  - [ ] Instrumentar o compilador para medir tempo de cada fase (lexer, parser, semântica, IR, otimização, codegen).
  - [ ] Adicionar comando/flag para mostrar estatísticas de tempo.
  - [ ] Usar profiler da plataforma (perf, VTune, etc.) para identificar gargalos.
- [ ] **Otimizações específicas**
  - [ ] **Lexer**:
    - [ ] Usar leitura em blocos e ponteiros em vez de cópias de string.
    - [ ] Minimizar alocações dinâmicas (usar `string_view`/equivalentes).
  - [ ] **Parser**:
    - [ ] Reduzir backtracking, simplificar gramática quando possível.
    - [ ] Evitar estruturas de dados pesadas para caminhos comuns.
  - [ ] **AST/Semântica**:
    - [ ] Reaproveitar nós/estruturas quando possível.
    - [ ] Evitar alocações excessivas por nó.
  - [ ] **LLVM**:
    - [ ] Ajustar conjunto de passes para equilibrar tempo de compilação e performance do código.
    - [ ] Expor modos rápidos (`-O0`) e modos agressivos (`-O2`, `-O3`).
- [ ] **Testes de carga**
  - [ ] Criar projetos grandes (muitos arquivos, muitas linhas) para teste de compilação.
  - [ ] Medir tempo de compilação em diferentes máquinas/plataformas.
  - [ ] Estabelecer metas de tempo (ex: compilar X linhas/segundo).

---

## Fase 10 – Ferramentas, DX (Developer Experience) e qualidade

- [ ] **CLI do compilador**
  - [ ] Implementar binário `vantorc` (ou nome da linguagem) com:
    - [ ] `vantorc arquivo.lang -o programa`
    - [ ] Suporte a múltiplos arquivos e pastas.
    - [ ] Flags de otimização (`-O0`, `-O1`, `-O2`, `-O3`).
    - [ ] Flags de depuração (`-g`, `--emit-llvm`, `--dump-ast`).
  - [ ] Implementar mensagens de ajuda (`--help`, `--version`).
- [ ] **Mensagens de erro amigáveis**
  - [ ] Implementar formato de erro com trecho de código e seta para a coluna.
  - [ ] Adicionar sugestões (`did you mean`) para identificadores similares.
  - [ ] Diferenciar erros, avisos e notas adicionais.
- [ ] **Integração com editores/IDE**
  - [ ] Planejar/implementar servidor LSP (Language Server Protocol).
  - [ ] Fornecer syntax highlight básico (extensão para VSCode, por exemplo).
  - [ ] Permitir execução/depuração rápida a partir do editor.
- [ ] **Testes automatizados e CI**
  - [ ] Configurar suíte de testes unitários para todas as fases.
  - [ ] Configurar testes de integração (compilar e rodar programas exemplo).
  - [ ] Configurar pipeline de CI (build + testes + lint).
- [ ] **Documentação**
  - [ ] Criar documentação da linguagem (sintaxe, tipos, exemplos).
  - [ ] Criar guia do compilador (flags, fluxo de compilação).
  - [ ] Criar exemplos reais de uso da linguagem (apps, libs pequenas).

---

## Roadmap incremental sugerido

// Estado atual do projeto em relação ao plano:
// - Lexer, Parser, AST e TypeChecker já existem e estão integrados.
// - CLI (`lex`) já faz build, run, bench e stress tests.
// - Backend LLVM inicial já gera IR textual com stub de `main` (arquivo `build/output.ll`).

- [ ] **Passo 1**: finalizar especificação mínima da linguagem (Fase 1).
- [x] **Passo 2**: implementar Lexer completo com testes (Fase 3).
- [x] **Passo 3**: implementar Parser + AST para um subconjunto da linguagem (Fase 4).
- [x] **Passo 4**: adicionar análise semântica básica (tipos primitivos e funções) (Fase 5).
- [x] **Passo 5**: integrar LLVM e gerar IR para expressões, funções simples e `main` (Fase 6) — **parcialmente concluído**: já gera IR textual com stub de `main`.
- [ ] **Passo 6**: criar runtime mínimo com `Console::log` e compilar um “Hello, world” (Fase 7).
- [ ] **Passo 7**: expandir semântica (tipos compostos, controle de fluxo completo) (Fase 5).
- [ ] **Passo 8**: introduzir modos de otimização (`-O0`, `-O2`) e medir performance (Fase 6 e 9).
- [ ] **Passo 9**: implementar sistema de log completo (compilador + runtime) (Fase 8).
- [ ] **Passo 10**: melhorar DX (CLI, mensagens de erro, integração com editor) e qualidade (Fase 10).


