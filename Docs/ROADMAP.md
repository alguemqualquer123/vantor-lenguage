# 🔮 Lexicon Language — Evolution Roadmap & Technical Strategy
### Advanced Language Specification — Full Edition

> **Versões (não confundir): linguagem v0.2.0** (workspace `Cargo.toml`)
> vs **extensão VS Code 1.1.x** (`themes/lexicon-vscode/package.json`).
> Status ✅ abaixo só onde o código comprova; o resto está em
> [Roadmap pós-0.2.0](#21-roadmap-pós-020).

---

## 1. Arquitetura e Componentes Core

### 1.1 Objetivos de Design

| Pilar | Descrição |
|---|---|
| **Expressividade** | Pipes (`▷`), Closures, Decoradores, Macros e Pattern Matching de primeira classe |
| **Produtividade** | Codegen automática (`@Getter`, `@Setter`), DI nativo, Hot Reload e REPL interativo |
| **Performance** | Backend LLVM com PGO, AOT compilation, SIMD nativo e interop C/Rust |
| **Segurança** | Affine Types, ownership semântico, effect system e proteção contra data races |
| **Corretude** | Proof system, refinement types, formal verification e testes de propriedade |
| **Ergonomia** | Inferência de tipos Hindley-Milner, erros amigáveis e zero boilerplate |

### 1.2 Ecossistema de Pastas

```text
lexicon/
├── lexicon-cli/          # CLI, Runner, REPL e Hot Reload (com suporte HTTP Real e Env)
├── lexicon-lexer/        # Tokenização (Pipes ▷, Interpolação, Unicode, Null-Safe 2.0)
├── lexicon-parser/       # AST, Decoradores, Macros e Anotações
├── lexicon-analysis/     # Type Checking, DI Engine, Borrow Checker e Semantic Analysis
├── lexicon-codegen/      # LLVM / WebAssembly / Native Code Generation
├── lexicon-optimizer/    # PGO, DCE, Inlining, SIMD Auto-vectorization
├── lexicon-mir/          # Mid-level IR para otimizações de alto nível
├── lexicon-vscode/       # Extensão VS Code (Syntax, LSP, Hover Docs, CodeLens Run/Debug)
├── lexicon-stdlib/       # Standard Library (I/O, Net, Collections, Async, Crypto)
├── lexicon-pkg/          # Package Manager e Registro de Pacotes
├── lexicon-ffi/          # Foreign Function Interface (C, Rust, Python, JS)
├── lexicon-proof/        # Proof Assistant e Formal Verification
├── lexicon-gpu/          # Kernels GPU (CUDA / Metal / Vulkan Compute)
└── lexicon-ai/           # Integração nativa com modelos de linguagem e ML
```

---

## 2. Sistema de Tipos Avançado

### 2.1 Tipos Algébricos (ADT)

Lexicon implementa Algebraic Data Types com pattern matching exaustivo e GADTs:

```lexicon
// Sum Types (Union Discriminada)
type Result<T, E> = Ok(T) | Err(E) | Pending

// Product Types com named fields
type Point = { x: Float64, y: Float64 }

// Recursive Types
type Tree<T> = Leaf | Node { value: T, left: Tree<T>, right: Tree<T> }

// GADTs — Generalized Algebraic Data Types
type Expr<T> =
    | Lit(Int)                          : Expr<Int>
    | Add(Expr<Int>, Expr<Int>)         : Expr<Int>
    | Eq(Expr<Int>, Expr<Int>)          : Expr<Bool>
    | If(Expr<Bool>, Expr<T>, Expr<T>)  : Expr<T>

// Pattern Matching exaustivo com guards
match result {
    Ok(val) if val > 0 => process(val),
    Ok(_)              => default(),
    Err(e)             => log_error(e),
    Pending            => retry(),
}
```

### 2.2 Sistema de Efeitos (Effect System)

Efeitos são rastreados e verificados em compile-time — elimina surpresas em I/O, concorrência e erros:

```lexicon
// Declaração de efeitos customizados
effect IO, Async, State<S>, Error<E>, Log, DB, Random

// Função com efeitos explícitos na assinatura
fn fetch_user(id: UserId) -> User ! [IO, Async, Error<NotFound>] {
    await http.get("/users/#{id}")?.parse::<User>()
}

// Handler de efeitos — intercepts em runtime tipado
handle fetch_user(42) {
    on Error<NotFound> => User.anonymous()
    on IO              => sandbox_io()
    on Log(msg)        => println("[TEST] #{msg}")
}

// Efeitos como rows — extensíveis e composáveis
fn run<E | IO>(f: Unit -> Unit ! [IO | E]) -> Unit ! E
```

### 2.3 Dependent Types & Refinement Types

Tipos podem depender de valores e carregar invariantes verificáveis em compile-time:

```lexicon
// Refinement Types — constraints inline
type PositiveInt              = Int    where self > 0
type NonEmptyString           = String where self.len() > 0
type BoundedList<T, N: Int>   = List<T> where self.len() <= N
type SortedVec<T: Ord>        = Vec<T>  where self.is_sorted()

// Divisão segura — verificada em compile-time, sem runtime panic
fn safe_div(a: Int, b: Int where b != 0) -> Int = a / b

// Vetores com tamanho no tipo (length-indexed vectors)
fn zip<T, U, N: Int>(a: Vec<T, N>, b: Vec<U, N>) -> Vec<(T, U), N>
fn head<T, N: Int where N > 0>(v: Vec<T, N>) -> T

// Type-level naturals
fn replicate<T, N: Nat>(value: T) -> Vec<T, N>
```

### 2.4 Ownership & Borrowing (Memory Safety sem GC)

```lexicon
// Ownership transfer
let s  = String.new("hello")   // s owns the data
let s2 = move s                 // ownership moved; s é inválido

// Borrow imutável — múltiplos leitores simultâneos
fn print_len(s: &String) -> Int = s.len()

// Borrow mutável — exclusivo, sem aliasing
fn append(s: &mut String, suffix: &str) { s.push(suffix) }

// Lifetimes inferidos automaticamente
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str =
    if x.len() >= y.len() { x } else { y }

// Smart pointers
let shared: Rc<Config>     = Rc.new(Config.default())
let atomic: Arc<Counter>   = Arc.new(Counter.new(0))
let boxed:  Box<dyn Trait> = Box.new(MyImpl {})

// Interior mutability
let cell: RefCell<Vec<Int>> = RefCell.new([])
cell.borrow_mut().push(42)
```

### 2.5 Linear Types & Session Types

Garantias de uso único e protocolos de comunicação verificados em compile-time:

```lexicon
// Linear Types — devem ser usados exatamente uma vez
linear type FileHandle = FileHandle(Int)

fn open(path: String) -> FileHandle       // cria handle
fn close(fh: FileHandle) -> Unit          // consome handle (obrigatório chamar)
fn read(fh: &FileHandle) -> String        // emprestado, não consome

// Session Types — protocolos de comunicação tipados
session type LoginProtocol =
    Send(Username)
    .Recv(Challenge)
    .Send(Response)
    .Recv(LoginProtocol)
    .End

fn authenticate(chan: Chan<LoginProtocol>) -> Bool {
    chan.send(get_username())
       .recv() |> solve_challenge()
       |> chan.send()
       .recv()
       .is_ok()
}
```

### 2.6 Inferência de Tipos Hindley-Milner Estendida

```lexicon
// Inferência total — sem anotações necessárias
let add = (a, b) => a + b           // inferido: (Int, Int) -> Int
let map = (f, xs) => xs.map(f)      // inferido: (A -> B, List<A>) -> List<B>

// Polimorfismo paramétrico
let identity = x => x               // inferido: forall A. A -> A

// Row polymorphism — duck typing estrutural tipado
fn greet(user: { name: String, .. }) = "Hello, #{user.name}"

greet({ name: "Alice", age: 30 })   // ok
greet({ name: "Bob", role: "admin" }) // ok — campos extras ignorados
```

---

## 3. Modelo de Concorrência Estruturada

### 3.1 Async/Await de Primeira Classe

```lexicon
// Tasks estruturadas com cancellation automático ao sair do scope
async fn process_all(ids: List<Id>) -> List<User> {
    scope {
        let tasks = ids.map(id => spawn fetch_user(id))
        tasks.await_all()
    }
}

// Timeout, retry e circuit breaker nativos
let result =
    fetch_user(42)
        .retry(max: 3, backoff: Exponential(base: 100.ms()))
        .timeout(5.seconds())
        .circuit_breaker(threshold: 5, window: 30.seconds())
        .await
```

### 3.2 Actors & Message Passing

```lexicon
actor Counter {
    state count: Int = 0

    receive Increment          => count += 1
    receive Decrement          => count -= 1
    receive Get(reply: Pid)    => reply ! count
    receive Reset              => count = 0
    receive AddMany(n: Int)    => count += n
}

// Supervisão e tolerância a falhas
supervisor CounterSup {
    strategy OneForOne
    children [
        { id: "counter", actor: Counter, restart: Always, max_restarts: 5 }
    ]
}
```

---

## 4. Metaprogramação Avançada

### 4.1 Macro System (Higiênico)

```lexicon
macro derive_debug(target: Struct) {
    impl Debug for #{target.name} {
        fn fmt(&self, f: &mut Formatter) -> String {
            let fields = #{
                target.fields
                    .map(f => `#{f.name}: {self.#{f.name}:?}`)
                    .join(", ")
            }
            "#{target.name} { #{fields} }"
        }
    }
}

@derive_debug
@derive_serialize
@derive_clone
struct Point { x: Float64, y: Float64 }
```

### 4.2 Compile-Time Computation (const eval)

```lexicon
const fn fibonacci(n: Int) -> Int =
    match n {
        0 | 1 => n
        _     => fibonacci(n-1) + fibonacci(n-2)
    }

const FIB_20: Int             = fibonacci(20)
const PRIMES: Array<Int, 100> = sieve_of_eratosthenes(100)
const ROUTES: RouteTable      = build_router(ROUTE_DEFINITIONS)
```

---

## 16. Roadmap Completo (Todas as Fases)

### Fase 1: Enterprise Ready ✅
| Feature | Status |
|---|---|
| `@Configuration`, `@Bean`, `@Test` | ✅ Concluído |
| Auto-Codegen (`@Getter`, `@Setter`, `@Data`) | ✅ Concluído |
| DI Engine com scopes e lazy injection | ✅ Concluído |

### Fase 2: Developer Experience ✅ (núcleo verificado; LSP/IDE via extensão 1.1.x)
| Feature | Status |
|---|---|
| Interactive CLI, REPL e Hot Reload (`lex run --watch`: supervisor com processo filho + debounce 300 ms + filtro só-`.lex` + backoff) | ✅ Concluído |
| LSP: Hover Docs e CodeLens Run/Debug (extensão VS Code 1.1.x) | ✅ Concluído (lado extensão) |
| Autocomplete contextual + auto-import, Go-to-Definition, Rename, gate de sintaxe pré-run (extensão 1.1.x) | ✅ Concluído (lado extensão) |
| Smart Strings, Null-Safe 2.0, Macros | ✅ Concluído (núcleo; sandbox procedural completa → pós-0.2.0) |

### Fase 3: Modern Platforms & Global Ecosystem ⚠️ (parcial — verificado item a item)
| Feature | Status |
|---|---|
| WASM (crate `lexicon-wasm` existe) com bindings JS automáticos | 📋 Pós-0.2.0 (bindings automáticos não verificados) |
| Lexicon Cloud e deploy serverless (`lex deploy` é stub simulado) | 📋 Pós-0.2.0 |
| FFI para C, Rust e Python (`lex ffi` gera stub, sem binding real) | 📋 Pós-0.2.0 |
| Portal Multilíngue (i18n) e Docs EN/PT (i18n da extensão 1.1.x: `package.nls.json` + `pt-br`) | ✅ Concluído (lado extensão) |
| Real Networking — servidor HTTP axum real + `Env::get` dinâmico no CLI (ver `demo-api/`) | ✅ Concluído |

### Fase 4: Advanced Type System 📋
| Feature | Status |
|---|---|
| ADT, GADTs e Pattern Matching exaustivo | 🔵 Em desenvolvimento |
| Effect System (IO, Async, Error, Custom) | 🔵 Planejado |
| Refinement Types e Dependent Types | 🔵 Planejado |
| Ownership, Borrow Checker e Lifetimes | 🔵 Planejado |
| Linear Types e Session Types | 🔵 Planejado |
| const eval e compile-time computation | 🔵 Planejado |

### Fase 5: Concurrent & Distributed Systems 📋
| Feature | Status |
|---|---|
| Structured Concurrency e Scoped Tasks | 📋 Planejado |
| Actor Model com supervisão e restart | 📋 Planejado |
| STM composável e lock-free | 📋 Planejado |
| Coroutines, Generators e Dataflow | 📋 Planejado |
| Distributed Runtime e RPC tipado | 📋 Planejado |
| Fork-join e parallel pipelines nativos | 📋 Planejado |

### Fase 6: Functional Programming 📋
| Feature | Status |
|---|---|
| Monads, Applicatives e do-notation | 📋 Planejado |
| Lazy evaluation e streams infinitos | 📋 Planejado |
| Lenses, Prisms e Optics | 📋 Planejado |
| Currying automático e composição | 📋 Planejado |
| List / map / async comprehensions | 📋 Planejado |
| Higher-Kinded Types e Monad Transformers | 📋 Planejado |

### Fase 7: Tooling & Ecosystem 📋
| Feature | Status |
|---|---|
| Package Manager com lockfile e workspace | 📋 Planejado |
| Build System incremental e distribuído | 📋 Planejado |
| Formatter, Linter e Debugger DAP | 📋 Planejado |
| Profiler e Flame Graphs | 📋 Planejado |
| Mensagens de erro amigáveis (Elm-style) | 📋 Planejado |
| Doctests executáveis | 📋 Planejado |
| Fuzzing automático integrado | 📋 Planejado |

### Fase 8: High Performance Computing 📋
| Feature | Status |
|---|---|
| GPU Kernels (CUDA / Metal / Vulkan) | 📋 Planejado |
| SIMD Auto-vectorization | 📋 Planejado |
| Arena Allocator e Memory Pools | 📋 Planejado |
| Zero-copy I/O e io_uring nativo | 📋 Planejado |
| Stack allocator e região de memória | 📋 Planejado |

### Fase 9: Formal Verification 📋
| Feature | Status |
|---|---|
| Proof System e teoremas em compile-time | 📋 Planejado |
| Property-based Testing nativo | 📋 Planejado |
| Design by Contract (`@pre`, `@post`) | 📋 Planejado |
| Model Checking para código concorrente | 📋 Planejado |
| Verificação de módulos críticos da stdlib | 📋 Planejado |

### Fase 10: AI & ML Integration 📋
| Feature | Status |
|---|---|
| Inferência de modelos ONNX/GGUF nativa | 📋 Planejado |
| DSL para pipelines de treinamento ML | 📋 Planejado |
| Embeddings e busca semântica nativa | 📋 Planejado |
| Autograd e diferenciação automática | 📋 Planejado |
| Geração de código assistida por LLMs | 📋 Planejado |

---

## 17. Padrões de Código e Convenções

| Elemento | Convenção | Exemplo |
|---|---|---|
| Classes / Traits | PascalCase | `class UserProfile`, `trait Serialize` |
| Decoradores | PascalCase | `@Configuration`, `@Bean`, `@Test` |
| Funções / Variáveis | snake_case | `fn calculate_total()`, `let user_name` |
| Constantes | SCREAMING_SNAKE | `const MAX_RETRY = 3` |
| Módulos / Pacotes | snake_case | `module auth.jwt` |
| Macros | snake_case`!` | `derive_debug!`, `json!`, `assert_eq!` |
| Tipos Genéricos | PascalCase param | `fn map<A, B>(f: A -> B)` |
| Lifetimes | lowercase letra | `'a`, `'static` |
| Efeitos | PascalCase | `effect IO`, `Async`, `State<S>` |
| Erros | PascalCase | `error NotFound { id: Int }` |
| Active Patterns | PascalCase | `active pattern Even(n)` |
| GPU Kernels | snake_case + `@gpu_kernel` | `@gpu_kernel fn matmul(...)` |
| Session Types | PascalCase | `session type LoginProtocol` |
| Provas / Teoremas | snake_case | `@prove theorem commutativity(...)` |

---

## 18. Standard Library — Módulos Completos

| Módulo | Responsabilidade |
|---|---|
| `std::io` | Arquivos, streams, buffers e pipes assíncronos |
| `std::net` | HTTP/2, WebSockets, TCP/UDP e DNS |
| `std::collections` | `Vec`, `HashMap`, `BTreeMap`, `Deque`, `PriorityQueue` |
| `std::concurrent` | Channels, `Mutex`, `RwLock`, `Semaphore`, pools |
| `std::crypto` | AES, RSA, ed25519, SHA-3, TLS nativo |
| `std::parser` | Combinadores monádicos e geração de lexers |
| `std::json` / `xml` / `toml` | Serde nativo com zero-copy e streaming |
| `std::test` | Unitários, property-based, fuzzing e mocking |
| `std::reflect` | Reflexão em runtime com acesso à AST de tipos |
| `std::math` | Álgebra linear, estatística e aritmética de precisão |
| `std::time` | Timestamps, durations, timezones e scheduling |
| `std::fs` | Filesystem com watch, permissões e paths seguros |
| `std::process` | Subprocessos, pipes e sinais |
| `std::env` | Variáveis de ambiente e argumentos CLI |
| `std::gpu` | Buffers GPU, dispatch de kernels e sincronização |
| `std::ml` | Tensores, autograd e operadores de ML |
| `std::log` | Logging estruturado com níveis e sinks |
| `std::metrics` | Contadores, histogramas e gauges exportáveis |
| `std::trace` | Spans, contextos de trace e exportadores OTLP |
| `std::proof` | Tipos de prova, táticas e lemmas |
| `std::ffi` | Bindings C, Rust, Python e JS automáticos |
| `std::wasm` | Runtime WASM embutido e interop com host |

---

## 19. Cronograma de Entrega

| Versão | Foco | Status |
|---|---|---|
| `v0.2-Alpha` | Decoradores & Pipes | ✅ Concluído |
| `v0.4-Alpha` | DI Engine & Auto-Codegen | ✅ Concluído |
| `v0.6-Beta` | Hot Reload (supervisor `--watch` ✅), LSP via extensão 1.1.x ✅, WASM auto-bindings 📋 | ✅ Parcial |
| `v0.8-Beta` | ADT, Effect System, Ownership, Linear & Session Types | 🔵 Em andamento |
| `v0.9-RC` | Concorrência Estruturada, Actors, STM & Coroutines | 📋 Planejado |
| `v1.0-Stable` | LLVM otimizado, Package Manager & Tooling completo | 📋 Planejado |
| `v1.2` | FP Avançado: Monads, Optics, Comprehensions & HKT | 📋 Planejado |
| `v1.5` | GPU, SIMD, Memory Pools & io_uring | 📋 Planejado |
| `v1.8` | Formal Verification, Proof System & Property Testing | 📋 Planejado |
| `v2.0` | AI/ML Integration, Distributed Runtime & Full Stdlib | 📋 Planejado |
| `v3.0` | Self-hosting: compilador Lexicon escrito em Lexicon | 📋 Planejado |

---

## 20. Critérios Globais de Sucesso

1. Compilação incremental < 100ms para projetos de médio porte.
2. Suporte a 100% das anotações de infraestrutura solicitadas.
3. Zero overhead no runtime para DI e traits monomorphizadas.
4. Tempo de startup do LSP < 500ms mesmo em projetos grandes.
5. Cobertura de 95%+ da stdlib por testes de propriedade.
6. Compatibilidade binária garantida entre versões minor (semver).
7. Memory safety verificada em compile-time — zero use-after-free.
8. Concorrência sem data races — verificada pelo type system.
9. Erros de compilação com sugestões de correção em 100% dos casos.
10. Performance de runtime dentro de 10% do equivalente em C/Rust.
11. Self-hosting: compilador Lexicon escrito em Lexicon na v3.0.
12. Formal verification de módulos críticos da stdlib (crypto, collections).
13. Kernels GPU com menos de 5% de overhead vs CUDA nativo.
14. Inferência de tipos em < 200ms para arquivos de até 5k linhas.

---

*Lexicon Language — Full Advanced Technical Specification · v2026*

---

## 21. Roadmap pós-0.2.0

Itens que este documento já marcava como concluídos mas o código
**não comprova** (comandos stub/simulados em `compiler.rs`: `deploy`
com barra de progresso + URL fixa, `ffi` sem binding real,
`generate` só lista call sites de macro, `publish` só valida
metadados, `debug` só lista símbolos), mais o já planejado das
Fases 4–10:

- Deploy real no Lexicon Cloud (hoje: `lex deploy` simulado).
- FFI real C/Rust/Python (hoje: `lex ffi` gera stub).
- WASM com bindings JS automáticos (hoje: só o crate existe).
- Sandbox procedural de macros (hoje: `lex generate` reporta sites).
- Debugger completo breakpoints/DWARF/PDB (hoje: inspeção de símbolos).
- Registry + `publish` com upload real (hoje: valida metadados).
- Fases 4–10 na íntegra (type system avançado, concorrência, FP,
  tooling/ecossistema, HPC, verificação formal, AI/ML).

Verificado e **feito** em v0.2.0 (não reabrir): supervisor
`--watch`, runner/lint/security comment-accurate via
`strip_comments`, installer silencioso idempotente com checagem de
registro, remoção da dep webview (link MinGW), super extensão VS Code
1.1.x (gramática/snippets/temas/comandos/autocomplete contextual/
diagnósticos/gate pré-run) e `demo-api/` (axum real + SQLite via
`lexicon-db` + `.env` dev/prod).
