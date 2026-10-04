# PLANO DE IMPLEMENTAÇÃO — Paridade Total com Go
## Lex/Lexicon tendo "tudo que Go tem", implementado na própria linguagem

> Origem do escopo: leitura integral de `C:\Program Files\Go` (distribuição Go 1.24).
> Documento vivo: cada fase marca o que já existe no Lex, o que falta, e o critério de aceite.

---

## 1. Inventário do Go (o que existe para replicar)

A distribuição Go tem **4 grandes blocos**:

| Bloco | Local em `C:\Program Files\Go` | Conteúdo |
|---|---|---|
| **Toolchain (cmd)** | `src/cmd/` | `go` (build/run/test/mod/...), `compile`, `link`, `asm`, `gofmt`, `vet`, `doc`, `test2json`, `cover`, `covdata`, `nm`, `objdump`, `addr2line`, `buildid`, `pack`, `fix`, `api`, `dist`, `pprof`, `trace`, `cgo` |
| **Runtime** | `src/runtime/` | Scheduler de goroutines, GC, canais/select, reflection, unsafe, métricas, pprof, trace, race detector |
| **Stdlib** | raiz de `src/` (48 pacotes) + subpacotes (~300 no total) | ver §2 |
| **Sistema de build** | `go.mod`/`go.sum`/`go.work`, module proxy | resolução de dependências, versionamento semântico, workspaces |

### 1.1 Stdlib completa por categoria (espelho fiel)

```
Core/esencial:     fmt strings strconv errors io bufio bytes time sort
                   math math/big math/bits math/cmplx math/rand
                   unicode unicode/utf8 unicode/utf16 cmp slices maps iter
                   container/heap container/list container/ring
                   path path/filepath io/ioutil io/fs embed
Sync/concorr.:     sync sync/atomic context runtime os/signal
                   (runtime: scheduler, GC, canais, select, defer já existe no Lex)
Sistema:           os os/exec os/user os/signal syscall log
                   flag expvar unique weak arena structs
Texto:             regexp regexp/syntax text/scanner text/tabwriter
                   text/template html html/template
Encoding:          encoding/json encoding/xml encoding/csv encoding/binary
                   encoding/base64 encoding/base32 encoding/hex encoding/pem
                   encoding/gob encoding/asn1 encoding/ascii85 mime mime/multipart
                   mime/quotedprintable
Net:               net net/http net/http/pprof net/url net/mail net/rpc
                   net/smtp net/textproto net/netip
Crypto:            crypto/aes crypto/cipher crypto/des crypto/dsa crypto/ecdh
                   crypto/ecdsa crypto/ed25519 crypto/elliptic crypto/hkdf
                   crypto/hmac crypto/md5 crypto/rand crypto/rc4 crypto/rsa
                   crypto/sha1 crypto/sha256 crypto/sha3 crypto/sha512
                   crypto/subtle crypto/tls crypto/x509 crypto/pbkdf2 ...
Hash:              hash hash/adler32 hash/crc32 hash/crc64 hash/fnv hash/maphash
Dados/mídia:       database/sql archive/tar archive/zip compress/flate
                   compress/gzip compress/zlib compress/lzw compress/bzip2
                   image image/color image/draw image/gif image/jpeg image/png
                   index/suffixarray debug/* (elf, pe, dwarf, macho, gosym, buildinfo)
Metaprogramação:   reflect unsafe builtin (tipos especiais)
Tooling p/ si:     go/ast go/parser go/token go/types go/constant go/doc
                   go/format go/printer go/scanner go/build go/version
Testes:            testing testing/quick testing/fstest testing/iotest
                   (fuzzing nativo, benchmarks, examples, cover)
```

---

## 2. Estado atual do Lex (v0.2.0) — o que já existe

| Componente | Estado | Onde |
|---|---|---|
| Lexer completo | ✅ | `src/lexicon-lexer/` |
| Parser/AST (imports, structs, enums, traits, genéricos na AST) | ✅ parse | `src/lexicon-parser/` |
| Análise semântica / type checker | ✅ parcial | `src/lexicon-analysis/` |
| Backend LLVM | ⚠️ stub (emite IR-texto) | `src/lexicon-codegen/` |
| **Interpretador real** (subconjunto executável) | ✅ | `src/lexicon-cli/src/interp.rs` |
| Builtins mock | ✅ | `Console`, `Json`, `Env`, `Http` (hardcoded em Rust) |
| Stdlib em Lex | ⚠️ 5 stubs (prelude, collections, json, http, testing) — **sem loader** | `build/sdk/lib/std/*.lex` |
| Módulos de domínio | ✅ | async, cache, db, gui, http, wasm, utils |
| CLI | ✅ | build/run/check/vet/fmt/test/sdk/lsp/repl/... |
| Defer | ✅ no interp | — |

### 2.1 Lacuna crítica identificada
O interpretador **não carrega `.lex` de verdade**: `import std::fmt` é ignorado em
`run_module` (interp.rs) e `fmt::X` cai em mock. Ou seja: **hoje não é possível
"ter tudo de Go" porque não existe loader de módulos**. Por isso a Fase 0 é pré-requisito
de tudo.

---

## 3. Estratégia geral

1. **Fase 0** libera a fundação: `import a::b` → `lib/a/b.lex` parseado e executado
   pelo interpretador real, com imports transitivos, `as`, detecção de ciclo.
2. **Fases 1–7** implementam a stdlib Go em Lex (`lib/std/...`), na ordem em que o
   interpretador consegue executar (crescendo o subconjunto da linguagem quando preciso).
3. **Fase 8** cresce o runtime: goroutines verdes, canais, select, GC simples, reflection.
4. **Fase 9** entrega o toolchain: `lex build/test/fmt/vet/doc/mod/...` com paridade
   funcional aos comandos `go`.
5. **Fase 10** faz o Lex olhar para si mesmo: `lex/ast`, `lex/parser`, `lex/types`
   (equivalentes a `go/ast`, `go/parser`, `go/types`).

Regra de ouro de cada pacote: **mesmos nomes e semântica da API Go**, adaptados ao
sintaxe Lex (e.g. Go `fmt.Println` → `fmt::Println`). Onde o Lex já tem builtin
(`Console::writeLine`), a stdlib Lex delega/encapsula.

---

## 4. Fases detalhadas

### FASE 0 — Loader de módulos Lex (fundação) ✅ esta sessão
- [ ] `Cx.mods: HashMap<seg, LoadedMod>` no interp; `call_module` consulta Lex antes do builtin Rust
- [ ] Resolve `std::fmt` → `<sdk>/lib/std/fmt.lex`; paths de busca: `./lib`, `./build/sdk/lib`, dir do exe, `LEX_PATH`
- [ ] Imports transitivos + visited-set; ciclo = erro claro
- [ ] Alias `import std::fmt as f`; chamadas `f::Println(...)` e `fmt.Println(...)`
- [ ] `pub` respeitado (itens não-pub de módulo externo = erro)
- **Aceite:** `import std::strings; fn main() { Console::log(strings::ToUpper("oi")); }` executa de verdade.

### FASE 1 — Núcleo executável (fmt, strings, strconv, errors, math, sort, time, unicode/utf8, bytes, io, bufio)
Entregáveis em `lib/std/` (esta sessão começa aqui):
- `errors`: `New`, `Is`, `As`, `Unwrap`, `Join` (modelo: struct `LexError{msg, cause}`)
- `strings`: `Contains`, `HasPrefix`, `HasSuffix`, `Split`, `Join`, `ToUpper`, `ToLower`,
  `Trim`, `TrimSpace`, `Replace`, `ReplaceAll`, `Index`, `Repeat`, `Fields`, `Builder`
- `strconv`: `Itoa`, `Atoi`, `FormatInt`, `ParseInt`, `FormatFloat`, `ParseFloat`, `Quote`, `Bool`
- `math`: `Abs`, `Max`, `Min`, `Pow`, `Sqrt`, `Floor`, `Ceil`, `Round`, `Trunc`, `Sign`,
  `Sin/Cos/Tan`, `Log`, `Exp`, `Mod`, `Pow10`, constantes `Pi`, `E`, `MaxInt`, `MinInt`
- `sort`: `Ints`, `Floats`, `Strings`, `Sort` (via índice+swap closure — requer função-valor no interp)
- `fmt`: `Print`, `Println`, `Printf` (verbos %v %s %d %f %t %x %%), `Sprintf`, `Errorf`, `Fprint*`
- `time`: `Now`, `Sleep` (ms), `Format` (layout básico), `Unix`, `Duration` (i64 ms)
- `unicode/utf8`: `RuneCount`, `ValidString`, `DecodeRune`
- `bytes`: `Compare`, `Contains`, `Index`, `Join`, `Repeat`, `ToUpper`, `ToLower`, `TrimSpace`
- `io`: `Reader`/`Writer` (structs com funções), `Copy`, `ReadAll`, `EOF` sentinel
- `bufio`: `Reader::readString/readLine`, `Writer`, `Scanner` (linhas/palavras)
- **Pré-requisitos de interp a adicionar se faltar:** função como valor (p/ comparators),
  método `char_at`/indexação de string, `for i in range` já existe.

### FASE 2 — Coleções & algoritmos genéricos
- `slices`: `Clone`, `Concat`, `Contains`, `Index`, `Sort`, `Reverse`, `Map` (função-valor), `Filter`, `Reduce`, `Compact`, `Insert`, `Delete`
- `maps`: `Keys`, `Values`, `Clone`, `Equal`, `Delete`, `Insert` (mapa = struct com pares, ou builtin map se houver)
- `cmp`: `Compare`, `Less`, `Greater`, `Or` (genéricos — AST já tem generics; interp ganha monomorfização por chamada)
- `container/list`: lista duplamente ligada (`List`, `Element`, `PushBack`, `PushFront`, `Remove`, `Len`, iterate)
- `container/heap`: heap binário sobre interface `Interface{Len,Less,Swap,Push,Pop}` (função-valor)
- `container/ring`: anel circular (`New`, `Next`, `Prev`, `Link`, `Unlink`, `Do`)
- `iter`: `Iterator` com `next()` e adaptadores (`Map`, `Filter`, `Take`, `Chain`)

### FASE 3 — Sistema, OS e concorrência
- `os`: `Args`, `Getenv`, `Setenv`, `Exit`, `Getwd`, `Chdir`, `Mkdir`, `MkdirAll`, `Remove`,
  `RemoveAll`, `ReadFile`, `WriteFile`, `ReadDir`, `Stat` (FileInfo), `Open`, `Create`,
  `Stdin/Stdout/Stderr`, `Hostname`, `TempDir`, `UserHomeDir`, `Environ`
- `path`: `Join`, `Split`, `Base`, `Dir`, `Ext`, `IsAbs`, `Clean` (separador `/`)
- `path/filepath`: idem com `\\` no Windows + `Walk`, `Glob`
- `io/fs`: `FS` interface, `WalkDir`, `Glob`, `ReadFileFS` (sobre o `os` real)
- `sync`: `Mutex` (interp: trava global por escopo), `RWMutex`, `WaitGroup`, `Pool`, `Once`, `Map`
- `sync/atomic`: `AddInt64`, `LoadInt64`, `StoreInt64`, `CompareAndSwap` (monomorfização)
- `context`: `Context` com `Deadline/Done/Err/Value`, `WithCancel`, `WithTimeout`, `WithValue`, `Background`, `TODO`
- `runtime`: `NumCPU`, `NumGoroutine`, `GOMAXPROCS`, `Version`, `GC` (estatísticas básicas do interp)
- `log`: `Print*`, `Fatal*`, `Panic*`, `SetPrefix`, `SetFlags` (timestamp), logger default
- `slog`: níveis estruturados (`Debug/Info/Warn/Error` + `With` attrs + JSON handler)
- `flag`: `String`, `Int`, `Bool`, `Parse`, `FlagSet`, `Usage`
- `expvar`: `Int`, `Float`, `String`, `Map`, `Publish`, handler HTTP `/debug/vars`
- `os/exec`: `Command` com `Run/Output/CombinedOutput`, pipes stdin/stdout
- `os/signal`: `Notify` (SIGINT/SIGTERM no runtime)
- `syscall`: nº de syscall por plataforma (stub documentado; cheio via builtin Rust)

### FASE 4 — Texto e encoding
- `regexp`: motor próprio: compila regex → AST → backtracking/NFA; `Match`, `Compile`,
  `MustCompile`, `Find`, `FindAll`, `FindSubmatch`, `ReplaceAll`, `Split`
  (começar por: literais, `.`, classes, `*+?`, grupos, `|`, âncoras, quantificadores `{n,m}`)
- `regexp/syntax`: AST de regex parseada (base do motor)
- `text/scanner`: tokenizer configurável (modo Go/Rust/etc.)
- `text/tabwriter`: tabelas alinhadas por tabulação
- `text/template`: templates `{{.Campo}} {{range}} {{if}} {{func}}` com execução segura
- `html`: `EscapeString`, `UnescapeString`
- `html/template`: template com auto-escape contextual
- `encoding/json`: `Marshal`, `Unmarshal`, `Encoder/Decoder`, `Valid`, tags de campo (struct→JSON real, hoje existe só builtin básico)
- `encoding/xml`: `Marshal`, `Unmarshal` com tags
- `encoding/csv`: `Reader/Writer` (RFC 4180)
- `encoding/base64`: `StdEncoding`, `URLEncoding`, `RawStdEncoding`
- `encoding/base32`, `encoding/hex`: idem
- `encoding/binary`: `Read/Write` big/little endian (u8..u64, f32/f64)
- `encoding/pem`: blocos `-----BEGIN-----`
- `encoding/gob` e `encoding/asn1`: binários estruturados (gob = formato próprio tipo pickle)
- `mime`: `ParseMediaType`, `TypeByExtension`, `ExtensionsByType`
- `mime/multipart`: formulários e e-mails MIME
- `path` já na Fase 3.

### FASE 5 — Rede
- `net`: `Dial`, `Listen`, `DialTimeout`, `Resolver`, `IP`, `TCPAddr/UDPAddr`, `Conn` interface
- `net/http`: **servidor+cliente reais**: `Get/Post/Head`, `Do`, `ServeMux`, `Handle/HandleFunc`,
  `Request/Response` structs, `Header`, `Status*`, middleware, `FileServer`, `StripPrefix`,
  (hoje existe mock `Http::get` — substituir por cliente real via builtin socket Rust)
- `net/url`: `Parse`, `URL{Scheme,Host,Path,Query}`, `QueryEscape`, `Values` (form data)
- `net/mail`: `ParseAddress`, `ReadMessage`
- `net/smtp`: `SendMail`, `PlainAuth`
- `net/textproto`: `Reader/Writer` de protocolos texto (HTTP, SMTP, NNTP por cima)
- `net/rpc`: RPC sobre HTTP+encoding (`Register`, `Call`)
- `net/netip`: `Addr`, `Prefix`, `ParseAddr` (IPv4/IPv6 sem alocação)

### FASE 6 — Criptografia
(Implementações puras em Lex sobre bytes; cheio de energia: sem deps externas.)
- `hash/*`: `crc32` (IEEE+Castagnoli), `crc64`, `adler32`, `fnv` (1/1a/64), `maphash`
- `crypto/md5`, `crypto/sha1`, `crypto/sha256`, `crypto/sha512`, `crypto/sha3` (Keccak)
- `crypto/hmac`, `crypto/pbkdf2`, `crypto/hkdf`
- `crypto/aes` (+ `cipher`: CBC, CTR, GCM), `crypto/des`
- `crypto/rc4`
- `crypto/rand`: bytes aleatórios do OS
- `crypto/rsa`, `crypto/ecdsa`, `crypto/ed25519`, `crypto/elliptic`: matemática bignum (`math/big` primeiro!)
- `crypto/x509`, `crypto/tls`: certificados + TLS 1.2/1.3 (fase avançada; pode delegar a builtin Rust no início)
- `crypto/subtle`: comparação em tempo constante

### FASE 7 — Dados, mídia, arquivos comprimidos
- `archive/zip`: `Reader/Writer`, CRC32 por entrada
- `archive/tar`: `Reader/Writer` (ustar/pax)
- `compress/flate`: DEFLATE (LZ77+Huffman) — coração de gzip/zlib/zip
- `compress/gzip`: cabeçalho+flate; `compress/zlib`: idem
- `compress/lzw`, `compress/bzip2` (leitura)
- `image`: `Image` interface (RGBA), `color`, `draw`, decodificadores `png`/`jpeg`/`gif`
- `database/sql`: `DB`, `Open`, `Prepare`, `Query/Exec`, `Rows`, `Tx`, drivers registráveis
  (driver SQLite já existe em Rust no `lexicon-db` — expor como driver Lex)
- `index/suffixarray`: índice de substring (algoritmo de Manber-Myers)
- `debug/elf`, `debug/pe`, `debug/macho`: parsers de binários (leitura de headers/seções)
- `debug/dwarf`, `debug/gosym`: tabelas de debug (avançado)

### FASE 8 — Runtime Go-parity
- **Goroutines**: threads verdes no interp (fila de tarefas cooperativa → preemptiva por steps);
  `spawn` (já existe na AST) passa a executar de verdade
- **Canais**: `chan T` com `send/recv`, buffer, fechamento, `select` com cases default/tempo
- **Scheduler**: work-stealing simples; `NumGoroutine`
- **GC**: contagem de refs do `Val` + ciclo de coleta quando memória > limiar (hoje tudo é `Clone`, GC = no-op documentado)
- **panic/recover**: `panic(msg)` unwinding até `recover()` (interp ganha exceção interna)
- **reflection**: `reflect` com `TypeOf`, `ValueOf`, leitura/escrita de campos e chamadas dinâmicas
- **unsafe**: `unsafe::SizeOf`, `OffsetOf` (documentado como no-op seguro no interp)
- **race detector**: log de corridas quando 2 goroutines tocam a mesma var sem lock (modo `--race`)

### FASE 9 — Toolchain (paridade com `cmd/go`)
- `lex build` → codegen LLVM real (hoje IR-texto) + `lex link` + `lex asm`
- `lex test` — framework completo: `test`、`lex test ./...`, `-run` regex, `-bench`, `-fuzz`, `-cover`
- `lex fmt` (existe — expandir para gofmt-parity: 100% dos programas idempotem)
- `lex vet` (existe — regras de go vet: printf, unreachable, struct tags)
- `lex doc` — extrai docs de `//` acima de `pub fn`, gera HTML/pkg.lex.dev
- `lex mod init/tidy/download/graph/verify` — `lex.mod`/`lex.sum` (go.mod/go.sum)
- `lex work` — workspaces (`lex.work`)
- `lex cover`/`lex pprof`/`lex trace` — perfis de cobertura/CPU/mem
- `lex playground` — servidor web de snippets
- `lex registry` — publicação de pacotes Lex (proxy de módulos)
- `lex fix` — reescritores automáticos de código antigo
- `lex test2json` — saída JSON para integração CI
- `lex nm`/`lex objdump`/`lex addr2line`/`lex buildid` — utilitários de binário

### FASE 10 — Tooling da própria linguagem (`go/*` → `lex/*`)
- `lex/ast`, `lex/token`, `lex/scanner`, `lex/parser`, `lex/types`, `lex/constant`,
  `lex/format` (gofmt interno), `lex/doc`, `lex/importer`, `lex/version`
- **Bootstrap meta**: reescrever o lexer/parser do Lex em Lex (self-host), compilando com o toolchain Lex
- LSP completo sobre `lex/ast` (hoje há entry `lex lsp` stub)

---

## 5. O que será entregue nesta sessão

- [x] Este plano (`Docs/PLANO_GO_FULL_PARITY.md`)
- [x] **Fase 0**: loader de módulos no `interp.rs` (já existia; validado
      ponta-a-ponta + correção: acesso a `mod::CONST`/`mod::global`)
- [x] **Fase 1 (núcleo)**: `errors`, `strings`, `strconv`, `math`, `sort`, `fmt`,
      `unicode/utf8`, `bytes`, `slices` (+Map/Filter/Reduce/SortFunc),
      `maps`, `cmp`, `container/list`, `container/heap`, `container/ring`,
      `iter`, `path`, `time`, `io`, `bufio` — em `lib/std/`, com testes `.lex`
- [x] **Fase 3 (parcial)**: `os`, `sync`, `context`, `log`, `path/filepath`
- [x] **Fase 4 (parcial)**: `encoding/json`, `encoding/base64`, `encoding/hex`,
      `encoding/csv`, `html`, `regexp` (subset documentado)
- [x] **Fase 6 (parcial)**: `hash/fnv`, `hash/crc32`, `hash/adler32`,
      `rand`, `crypto/subtle`
- [x] Upgrades do interpretador (velocidade + capacidade): lambdas como valor
      (`|x| ...`, inclusive zero-arg `| | x`), chamada de comparador via campo
      (`h.less(a, b)`), builtins nativos `Math::*`/`Text::*`/`List::sort_*`/
      `Hash::*`/`Rand::*`/`Json::valid`, fast-path i64 exato (sem round-trip
      f64 — bit-ops corretas além de 2^53)
- [x] SDK exporter atualizado p/ embutir os novos pacotes (40 arquivos)
- [x] Testes de ponta-a-ponta rodando via `lex test`/`cargo test`
      (e2e 130/130 + `cargo test -p lexicon-parser/cli` verdes)

## 6. Segunda sessão (v0.3.0) — velocidade + mais paridade

- [x] **Runtime mais rápido**: ASCII fast paths em `Text::slice/code_at/len`
      (regexp −31%), append in-place `x = x + y` (O(n²)→O(n)), tabelas de
      funções/structs em `Rc` (sem clone profundo por chamada)
- [x] **Pipeline**: sleeps falsos removidos de `lex ffi`/`lex deploy`
      (~1s/2.3s → ms); progress bars ocultas em CI/non-TTY
- [x] **UTF-8 de verdade**: `strip_comments`/`mask_mock_strings` reescritos
      char-based (antes cada byte virava um char); BOM (U+FEFF) aceito;
      `unsafe::` como chave de módulo no parser
- [x] **Novos nativos**: `Process::output`, `Atomic::*`, `Hash::crc64/maphash`,
      `Sys::{goos,arch,ncpu}`
- [x] **Novos pacotes**: `os/exec`, `os/signal`, `os/user`, `sync/atomic`,
      `net/url`, `net/mail`, `net/textproto`, `mime`, `mime/multipart`,
      `slog`, `flag`, `expvar`, `encoding/base32`, `encoding/ascii85`,
      `encoding/pem`, `math/bits`, `unicode/utf16`, `image/color`,
      `archive/tar`, `hash/crc64`, `hash/maphash`, `crypto/sha256` (FIPS),
      `crypto/hmac` (RFC 4231), `runtime`, `testing`, `unsafe`
- [x] **Extras**: `errors::{Wrap,Unwrap,Join}` + `Is` recursivo,
      `fmt::Errorf`, `slices::{Repeat,Chunk}`, `time::{UnixMicro,ParseDate}`,
      `strconv::Unquote`, `regexp::QuoteMeta`,
      `strings::{Cut,CutPrefix,CutSuffix,Trim,TrimLeft,TrimRight}`
- [x] **Extensão atualizada**: `complete.rs` rebuilt sobre a superfície real
      (13 nativos + 61 pacotes `std::` + keywords verdadeiras do lexer);
      `lex ide` com gramática TextMate + comando `extension`
      (`editors/vscode-lex` instalável, zero-build); `.vscode/` regenerado
- [x] Versão do binário 0.2.0 → **0.3.0** (Cargo, banner, fingerprint, SBOM,
      `runtime::Version`, CHANGELOG); e2e **139/139**- [x] **Instalação global + SDK auto**: `lex install` instala binário + SDK
      (`~/.lexicon`, Windows/macOS/Linux); todo run silencioso instala o SDK
      na primeira vez e atualiza no drift de `VERSION`; loader resolve
      `~/.lexicon/sdk/lib`; Unix com `chmod +x`, fish/`.zprofile`,
      self-update anti-ETXTBSY, `LEX_PATH` por plataforma

## 7. Go-to-definition (v0.3.1)

- [x] `textDocument/definition` no LSP com `definitionProvider: true`
- [x] `lib/std/native/*.lex`: 14 stubs de assinatura dos builtins nativos
      (navegação no IDE; import aborta em vez de silenciar)
- [x] Resolução espelha as raízes do interpretador (mesmo SDK de execução)
- [x] Versão do binário **0.3.1** (dispara o auto-update do SDK instalado)

### 5.1 Decisões técnicas da Fase 0
| Decisão | Escolha | Motivo |
|---|---|---|
| Chave do módulo | último segmento do path (`std::strings` → `strings`) | bate com a sintaxe de chamada `strings::X` |
| Alias | `import std::fmt as f` → chave = `f` | igual a Rust/Go |
| Fallback | Lex stdlib primeiro; builtin Rust depois | retrocompatível com `core::io::Console` |
| Raiz da stdlib | `./lib`, `./build/sdk/lib`, `<exe>/lib`, `<exe>/../lib`, `$LEX_PATH` | funciona em dev e no SDK exportado |
| Ciclo | erro `import cycle: a -> b -> a` | igual Go |
| `pub` | não-pub em módulo externo → erro de compilação | igual Go (exported vs unexported) |

### 5.2 Regras de escrita da stdlib em Lex (descobertas desta sessão)
| Regra | Detalhe |
|---|---|
| Chamadas sempre qualificadas | `strings::Index`, nunca `Index` — o namespace achatado resolve para o primeiro módulo carregado |
| Structs com nomes globalmente únicos | literais `Nome {...}` resolvem pelo mapa achatado (`Reader` do `io` vs `bufio` → `BufReader`; `PopResult` → `ListPopResult`/`HeapPopResult`) |
| Sem `campo < campo` | `a.ms < b.ms` parseia `<` como type-args; usar locais (`let x = a.ms;`) ou `<=`/`>`/`==` |
| Sem `\|` infixo | `x \| y` parseia como pipe-closure; usar `^`/`&`/`+` ou reestruturar |
| Lambdas `\|x\| ...` (standalone) e `\| \| x` (zero args, com espaços) | `||` sem espaço lexes como `OrOr` |
| Value semantics | structs/listas passam por cópia — ops que avançam retornam o valor (`r = io::Read(r, n).reader`, `xs = sort::Ints(xs)`) |
| Comparadores como lambda | `heap::New(\|a, b\| a < b)`; chamada via campo funciona (`h.less(a, b)`) |
