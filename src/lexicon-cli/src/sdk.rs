//! Lexicon SDK packager (`lex sdk`).
//!
//! Assembles a redistributable SDK tree (no new repo folders: the output
//! defaults to the existing `build/sdk/` directory):
//!
//! ```text
//! sdk/
//!   VERSION README.md LICENSE-MIT.md
//!   bin/lex(.exe)            <- copy of the running toolchain binary
//!   lib/std/*.lex            <- Lex standard-library sources
//!   templates/{default,api,service}/  <- `lex new` starters + manifest
//!   examples/*.lex
//!   docs/{CLI,STDLIB,EMBEDDING}.md
//!   scripts/{activate.ps1,activate.sh}
//! ```
//!
//! `export` copies the *running* binary, so build the flavour you want to
//! ship first: `cargo build --profile dist --no-default-features -p lexicon-cli`
//! for the ~3.5 MB mini runtime, or default features for the full IDE build.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(windows)]
const EXE_NAME: &str = "lex.exe";
#[cfg(not(windows))]
const EXE_NAME: &str = "lex";

/// `true` for full builds (`lex gui` available), `false` for `--no-default-features`.
pub const HAS_GUI: bool = cfg!(feature = "gui");

fn sdk_root(out: &Option<String>) -> PathBuf {
    match out {
        Some(o) => PathBuf::from(o),
        None => PathBuf::from("build").join("sdk"),
    }
}

fn write_file(root: &Path, rel: &str, content: &str, written: &mut Vec<(String, u64)>) -> Result<()> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating dir {}", parent.display()))?;
    }
    fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
    written.push((rel.to_string(), content.len() as u64));
    Ok(())
}

// ---------------------------------------------------------------------------
// Standard library sources (must pass `lex check`; validated on export)
// ---------------------------------------------------------------------------

// Go-parity packages: single source of truth in the repo `lib/std/` tree,
// embedded here so `sdk export` ships exactly the tested sources.
const STD_STRINGS: &str = include_str!("../../../lib/std/strings.lex");
const STD_STRCONV: &str = include_str!("../../../lib/std/strconv.lex");
const STD_MATH: &str = include_str!("../../../lib/std/math.lex");
const STD_SORT: &str = include_str!("../../../lib/std/sort.lex");
const STD_SLICES: &str = include_str!("../../../lib/std/slices.lex");
const STD_ERRORS: &str = include_str!("../../../lib/std/errors.lex");
const STD_PATH: &str = include_str!("../../../lib/std/path.lex");
const STD_FMT: &str = include_str!("../../../lib/std/fmt.lex");
// Fase 1 (núcleo restante): tempo, bytes, fluxos, unicode, mapas, comparação.
const STD_TIME: &str = include_str!("../../../lib/std/time.lex");
const STD_BYTES: &str = include_str!("../../../lib/std/bytes.lex");
const STD_IO: &str = include_str!("../../../lib/std/io.lex");
const STD_BUFIO: &str = include_str!("../../../lib/std/bufio.lex");
const STD_MAPS: &str = include_str!("../../../lib/std/maps.lex");
const STD_CMP: &str = include_str!("../../../lib/std/cmp.lex");
const STD_ITER: &str = include_str!("../../../lib/std/iter.lex");
const STD_UTF8: &str = include_str!("../../../lib/std/unicode/utf8.lex");
// Fase 2 (coleções): contêineres.
const STD_LIST: &str = include_str!("../../../lib/std/container/list.lex");
const STD_HEAP: &str = include_str!("../../../lib/std/container/heap.lex");
const STD_RING: &str = include_str!("../../../lib/std/container/ring.lex");
// Fase 3 (sistema): os, sync, context, log, filepath.
const STD_OS: &str = include_str!("../../../lib/std/os.lex");
const STD_SYNC: &str = include_str!("../../../lib/std/sync.lex");
const STD_CONTEXT: &str = include_str!("../../../lib/std/context.lex");
const STD_LOG: &str = include_str!("../../../lib/std/log.lex");
const STD_FILEPATH: &str = include_str!("../../../lib/std/path/filepath.lex");
// Fase 4 (texto/encoding): json, base64, hex, csv, html, regexp.
const STD_ENCODING_JSON: &str = include_str!("../../../lib/std/encoding/json.lex");
const STD_BASE64: &str = include_str!("../../../lib/std/encoding/base64.lex");
const STD_HEX: &str = include_str!("../../../lib/std/encoding/hex.lex");
const STD_CSV: &str = include_str!("../../../lib/std/encoding/csv.lex");
const STD_HTML: &str = include_str!("../../../lib/std/html.lex");
const STD_REGEXP: &str = include_str!("../../../lib/std/regexp.lex");
// Fase 6 (hash/aleatório): fnv, crc32, adler32, rand, subtle.
const STD_FNV: &str = include_str!("../../../lib/std/hash/fnv.lex");
const STD_CRC32: &str = include_str!("../../../lib/std/hash/crc32.lex");
const STD_ADLER32: &str = include_str!("../../../lib/std/hash/adler32.lex");
const STD_RAND: &str = include_str!("../../../lib/std/rand.lex");
const STD_SUBTLE: &str = include_str!("../../../lib/std/crypto/subtle.lex");
// Segunda onda (sistema/rede/texto/crypto): exec, atomic, url, slog, flag,
// mime, base32, bits, sha256, hmac.
const STD_EXEC: &str = include_str!("../../../lib/std/os/exec.lex");
const STD_ATOMIC: &str = include_str!("../../../lib/std/sync/atomic.lex");
const STD_URL: &str = include_str!("../../../lib/std/net/url.lex");
const STD_SLOG: &str = include_str!("../../../lib/std/slog.lex");
const STD_FLAG: &str = include_str!("../../../lib/std/flag.lex");
const STD_MIME: &str = include_str!("../../../lib/std/mime.lex");
const STD_BASE32: &str = include_str!("../../../lib/std/encoding/base32.lex");
const STD_BITS: &str = include_str!("../../../lib/std/math/bits.lex");
const STD_SHA256: &str = include_str!("../../../lib/std/crypto/sha256.lex");
const STD_HMAC: &str = include_str!("../../../lib/std/crypto/hmac.lex");
// Terceira onda (Go-parity): unsafe, runtime, testing, mail, textproto,
// multipart, pem, ascii85, color, tar, crc64, maphash, signal, user,
// expvar, utf16.
const STD_UNSAFE: &str = include_str!("../../../lib/std/unsafe.lex");
const STD_RUNTIME: &str = include_str!("../../../lib/std/runtime.lex");
const STD_MAIL: &str = include_str!("../../../lib/std/net/mail.lex");
const STD_TEXTPROTO: &str = include_str!("../../../lib/std/net/textproto.lex");
const STD_MULTIPART: &str = include_str!("../../../lib/std/mime/multipart.lex");
const STD_PEM: &str = include_str!("../../../lib/std/encoding/pem.lex");
const STD_ASCII85: &str = include_str!("../../../lib/std/encoding/ascii85.lex");
const STD_COLOR: &str = include_str!("../../../lib/std/image/color.lex");
const STD_TAR: &str = include_str!("../../../lib/std/archive/tar.lex");
const STD_CRC64: &str = include_str!("../../../lib/std/hash/crc64.lex");
const STD_MAPHASH: &str = include_str!("../../../lib/std/hash/maphash.lex");const STD_SIGNAL: &str = include_str!("../../../lib/std/os/signal.lex");
const STD_USER: &str = include_str!("../../../lib/std/os/user.lex");
const STD_EXPVAR: &str = include_str!("../../../lib/std/expvar.lex");
const STD_UTF16: &str = include_str!("../../../lib/std/unicode/utf16.lex");
// Native signature stubs for IDE go-to-definition (`native/*.lex`).
const STD_NATIVE_CONSOLE: &str = include_str!("../../../lib/std/native/console.lex");
const STD_NATIVE_JSON: &str = include_str!("../../../lib/std/native/json.lex");
const STD_NATIVE_ENV: &str = include_str!("../../../lib/std/native/env.lex");
const STD_NATIVE_HTTP: &str = include_str!("../../../lib/std/native/http.lex");
const STD_NATIVE_TIME: &str = include_str!("../../../lib/std/native/time.lex");
const STD_NATIVE_FILE: &str = include_str!("../../../lib/std/native/file.lex");
const STD_NATIVE_PROCESS: &str = include_str!("../../../lib/std/native/process.lex");
const STD_NATIVE_TEXT: &str = include_str!("../../../lib/std/native/text.lex");
const STD_NATIVE_MATH: &str = include_str!("../../../lib/std/native/math.lex");
const STD_NATIVE_LIST: &str = include_str!("../../../lib/std/native/list.lex");
const STD_NATIVE_HASH: &str = include_str!("../../../lib/std/native/hash.lex");
const STD_NATIVE_RAND: &str = include_str!("../../../lib/std/native/rand.lex");
const STD_NATIVE_ATOMIC: &str = include_str!("../../../lib/std/native/atomic.lex");
const STD_NATIVE_SYS: &str = include_str!("../../../lib/std/native/sys.lex");

const STD_PRELUDE: &str = r#"// Lexicon Standard Library — prelude.
// Core sum types and assertion helpers. Import by copying into your project
// (a module loader is on the roadmap).

enum Option<T> {
    Some,
    None,
}

enum Result<T, E> {
    Ok,
    Err,
}

pub fn assert_true(cond: bool) -> bool {
    return cond;
}

pub fn assert_eq_int(a: i32, b: i32) -> bool {
    return a == b;
}

pub fn identity_int(x: i32) -> i32 {
    return x;
}
"#;

const STD_COLLECTIONS: &str = r#"// Lexicon Standard Library — collections.
// Fixed-capacity Stack and Queue sketches over i32.

struct IntStack {
    top: i32,
}

struct IntQueue {
    head: i32,
    tail: i32,
}

pub fn stack_new() -> IntStack {
    return IntStack { top: 0 };
}

pub fn stack_empty(s: IntStack) -> bool {
    return s.top == 0;
}

pub fn queue_new() -> IntQueue {
    return IntQueue { head: 0, tail: 0 };
}

pub fn queue_empty(q: IntQueue) -> bool {
    return q.head == q.tail;
}
"#;

const STD_JSON: &str = r#"// Lexicon Standard Library — json.
// Minimal JSON value model plus stringify helpers.

enum JsonKind {
    Null,
    Bool,
    Number,
    Text,
    Array,
    Object,
}

struct JsonDoc {
    kind: JsonKind,
}

pub fn json_null() -> JsonDoc {
    return JsonDoc { kind: JsonKind::Null };
}

pub fn json_ok() -> String {
    return "{\"success\":true}";
}

pub fn json_error(msg: String) -> String {
    return msg;
}
"#;

const STD_HTTP: &str = r#"// Lexicon Standard Library — http.
// Route/handler vocabulary for `Http::serve` programs.

enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
}

struct Route {
    method: HttpMethod,
    path: String,
}

pub fn route_count_hint() -> i32 {
    return 0 as i32;
}

pub fn default_port() -> i32 {
    return 3000 as i32;
}
"#;

// Real `testing` package (Go-parity `T`/assertions/benchmark): single
// source of truth in `lib/std/testing.lex`, shipped exactly as tested.
const STD_TESTING: &str = include_str!("../../../lib/std/testing.lex");

// ---------------------------------------------------------------------------
// Templates (mirror `lex new` outputs so SDK == toolchain behaviour)
// ---------------------------------------------------------------------------

const TPL_DEFAULT_MAIN: &str = r#"pub fn main() -> void {
    Console::writeLine("Hello, Lexicon!");
}
"#;

const TPL_API_MAIN: &str = r#"@Get("/")
pub fn hello() -> String {
    return "Hello from Lexicon REST API!";
}

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
"#;

const TPL_SERVICE_MAIN: &str = r#"struct User { id: i32, name: String }

service UserService {
    rpc GetUser(id: i32) -> User;
}

pub fn main() -> void {
    Grpc::serve(UserService, "0.0.0.0:50051");
}
"#;

fn manifest(name: &str, template: &str) -> String {
    format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\ntemplate = \"{}\"\nedge = false\ntarget = \"native\"\ngrpc = false\n\n[dependencies]\ncore = \"0.1.0\"\n",
        name, template
    )
}

// ---------------------------------------------------------------------------
// Examples + docs + scripts
// ---------------------------------------------------------------------------

const EX_HELLO: &str = r#"// SDK example: hello.lex — run with `lex run hello.lex`
pub fn main() -> void {
    Console::writeLine("Hello from the Lexicon SDK!");
}
"#;

const EX_API: &str = r#"// SDK example: api.lex — run with `lex run api.lex`, then open
// http://localhost:3000/ (real axum server with CORS + JSON envelopes).
@Get("/")
pub fn hello() -> String {
    return "Hello from Lexicon REST API!";
}

@Get("/health")
pub fn health() -> String {
    return "ok";
}

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
"#;

const DOC_CLI: &str = r#"# Lexicon CLI Reference (SDK)

Generated for the SDK. Run `lex <cmd> --help` for flags.

| Command | Purpose |
|---|---|
| `lex run [file] [--watch] [--ci]` | Compile + run (hot reload with `--watch`) |
| `lex build [file] [--release]` | Compile to binary / LLVM IR |
| `lex check [file]` | Type-check without building |
| `lex test [--verbose]` | Run `@Test` suites (in-process batching) |
| `lex bench` / `lex benchmark` | Micro-benchmarks |
| `lex fmt [--check]` | Format sources |
| `lex lint [--json]` | Static lint rules |
| `lex vet [file]` | Formal verification checks |
| `lex doc [file]` | Generate documentation |
| `lex trace [file]` | Execution tracing |
| `lex profile [file]` | CPU/memory profile |
| `lex debug [file]` | Debug adapter |
| `lex generate` | Code generation (macro sites) |
| `lex mod` | Module/dependency manifest validation |
| `lex env` | Toolchain environment |
| `lex version` | Version info |
| `lex clean` | Remove build artifacts |
| `lex publish [--dry-run]` | Publish package |
| `lex new <name> [--template]` | Scaffold project (api, plugin, service) |
| `lex init` | Init project in cwd |
| `lex install` / `lex uninstall` | Global PATH install |
| `lex deploy [--env]` | Cloud deploy |
| `lex ffi <lib>` | C/Rust bindings |
| `lex gui [file]` | Native GUI IDE (full builds only) |
| `lex repl` | Interactive REPL |
| `lex visualize <file>` | AST/CFG visualization |
| `lex sdk [export\|verify\|info]` | SDK packaging (this kit) |
| `lex fix [file] [--dry-run]` | Auto-migration rewrites (dot-call canon, ws trim) |
| `lex complete` / `lex lsp` / `lex ide` | Autocomplete engine, language server, editor assets |

Non-interactive use: pass `--ci` or set `CI=true` (skips prompts and HTTP serve).
"#;

const DOC_STDLIB: &str = r#"# Lexicon Standard Library (SDK `lib/std/`)

Core kit (copy into your project):

| Module | Contents |
|---|---|
| `prelude.lex` | `Option<T>` / `Result<T,E>` enums, assert helpers |
| `collections.lex` | `IntStack` / `IntQueue` structs + constructors |
| `json.lex` | `JsonKind` enum, `JsonDoc`, stringify helpers |
| `http.lex` | `HttpMethod` enum, `Route` struct, port defaults |
| `testing.lex` | Equality/truth helpers for `@Test` suites |

Go-parity packages (loadable via `import std::<name>;`):

| Module | Go counterpart | Contents |
|---|---|---|
| `strings.lex` | `strings` | search, split/join, case, trim, rune count |
| `strconv.lex` | `strconv` | int/float/bool parse + format, Quote |
| `math.lex` | `math` | constants, roots, exp/log/pow, round, clamp |
| `sort.lex` | `sort` | sort ints/floats/strings (returns new list), search, is-sorted |
| `slices.lex` | `slices` (Go 1.21) | index/contains, clone, reverse, min/max, insert/delete |
| `errors.lex` | `errors` | `Error` struct, New/Is/Message |
| `path.lex` | `path` (POSIX) | join, split, dir/base, ext, clean, is-abs |
| `fmt.lex` | `fmt` | Print/Println/Printf, Sprint/Sprintln/Sprintf |
| `time.lex` | `time` | Time/Now/Sleep/Duration-ms, Format layouts |
| `bytes.lex` | `bytes` | byte-list compare/search/join/split/case |
| `io.lex` | `io` | Reader/Writer values, Read/ReadAll/Write/Copy |
| `bufio.lex` | `bufio` | BufReader, ReadString, line/word Scanner |
| `maps.lex` | `maps` | pair-list maps: Get/Set/Delete/Keys/Merge |
| `cmp.lex` | `cmp` | CompareInt/Float/String, Less, Or, Min/Max |
| `iter.lex` | `iter` | pull Iter + Map/Filter/Reduce/Take/Chain |
| `unicode/utf8.lex` | `unicode/utf8` | RuneCount/Decode/Encode/ValidString |
| `container/list.lex` | `container/list` | List, PushBack/Front, Pop, Remove |
| `container/heap.lex` | `container/heap` | Heap with lambda `less`, Push/Pop/Peek |
| `container/ring.lex` | `container/ring` | Ring cursor: Next/Prev/Set/Do |
| `os.lex` | `os` | Args/env/files/dirs via native builtins |
| `sync.lex` | `sync` | Mutex/RWMutex/WaitGroup/Once (coop. no-op) |
| `context.lex` | `context` | Background/WithCancel/Timeout/Value/Done |
| `log.lex` | `log` | default logger, prefix/flags, Fatal/Panic |
| `path/filepath.lex` | `path/filepath` | OS separators, Walk, glob Match |
| `encoding/json.lex` | `encoding/json` | Marshal/Unmarshal/Valid (native) |
| `encoding/base64.lex` | `encoding/base64` | Std/URL/Raw encode + Decode |
| `encoding/hex.lex` | `encoding/hex` | lowercase hex Encode/Decode |
| `encoding/csv.lex` | `encoding/csv` | RFC-4180 ReadAll/WriteAll |
| `html.lex` | `html` | EscapeString/UnescapeString |
| `regexp.lex` | `regexp` | Match/Find/ReplaceAll/Split (subset) |
| `hash/fnv.lex` | `hash/fnv` | FNV-1a 32/64 one-shot + streaming |
| `hash/crc32.lex` | `hash/crc32` | IEEE Checksum + streaming |
| `hash/adler32.lex` | `hash/adler32` | Checksum + streaming |
| `rand.lex` | `math/rand` | Seed/Intn/Float64/Shuffle/Choice |
| `crypto/subtle.lex` | `crypto/subtle` | ConstantTimeCompare/Select |
| `os/exec.lex` | `os/exec` | Command/Output/Run/LookPath (native spawn) |
| `sync/atomic.lex` | `sync/atomic` | Int64 cells: Load/Add/Swap/CAS (native) |
| `net/url.lex` | `net/url` | Parse/StringOf/QueryEscape/EncodeQuery |
| `slog.lex` | `slog` | levels, With attrs, default Dbg/Inf |
| `flag.lex` | `flag` | String/Int/Bool/Parse/Getters/Usage |
| `mime.lex` | `mime` | ParseMediaType/Format/TypeByExtension |
| `encoding/base32.lex` | `encoding/base32` | Std/Hex Encode + Decode |
| `math/bits.lex` | `math/bits` | OnesCount/zeros/rotate/reverse |
| `crypto/sha256.lex` | `crypto/sha256` | Sum/SumBytes (FIPS vectors) |
| `crypto/hmac.lex` | `crypto/hmac` | HMAC-SHA256 (RFC 4231 vector) |
| `unsafe.lex` | `unsafe` | Sizeof/Alignof/IsNil (safe subset) |
| `runtime.lex` | `runtime` | GOOS/GOARCH/NumCPU/Version (native) |
| `net/mail.lex` | `net/mail` | ParseAddress/ParseAddressList |
| `net/textproto.lex` | `net/textproto` | ReadMIMEHeader/CanonicalKey/Get |
| `mime/multipart.lex` | `mime/multipart` | Parse/HeaderGet/FileName |
| `encoding/pem.lex` | `encoding/pem` | Decode/Encode blocks |
| `encoding/ascii85.lex` | `encoding/ascii85` | z/y shortcuts Encode/Decode |
| `image/color.lex` | `image/color` | RGBA/NRGBA/Gray/ParseHex/ToHex |
| `archive/tar.lex` | `archive/tar` | ustar AppendFile/Next |
| `hash/crc64.lex` | `hash/crc64` | ECMA Checksum + streaming |
| `hash/maphash.lex` | `hash/maphash` | Seeded String/Bytes/Int |
| `os/signal.lex` | `os/signal` | constants + Notify/Stop/Wanted |
| `os/user.lex` | `os/user` | Current/Lookup via env |
| `expvar.lex` | `expvar` | NewInt/Add/Set/Get/Render JSON |
| `unicode/utf16.lex` | `unicode/utf16` | surrogate Encode/Decode |
| `native/*.lex` (14 files) | IDE stubs | Ctrl+Click targets for native builtins (`Math::sqrt` → `native/math.lex`); never import (bodies abort) |

All files pass `lex check`. Cross-package calls must stay package-qualified
(`strings::Index`, not bare `Index`) — the flattened namespace resolves
same-named functions to the first-loaded module.
"#;

const DOC_EMBEDDING: &str = r#"# Embedding Lexicon (SDK)

v1 integration contract is the stable CLI over subprocess:

- `lex check <file>` — exit 0 + diagnostics on stdout; non-zero on errors.
- `lex run --ci <file>` — runs to completion, never prompts, never serves.
- `lex test` — prints `Passed: N / Failed: M` summary lines (parse them).
- `lex fmt --check` — exit status only (0 = clean).
- `lex version` — `lex <ver> (lexc <ver>, spec v0.1)` first line.

Environment knobs honoured by every binary:

- `CI=true` / `--ci` — non-interactive mode.
- `LEXICON_NO_AUTO_INSTALL=1` — skip global auto-install copy.
- `LEX_SUPERVISED=1` — set by `lex run --watch` on supervised children.
- `RUST_LOG=debug` — verbose toolchain logging.

A native C ABI (`lex.h` + cdylib) is planned; until then drive `bin/lex`
and parse its stable stdout contracts above.
"#;

const SDK_README: &str = r#"# Lexicon SDK

Self-contained LexiconLang toolchain kit.

- `bin/` — the `lex` compiler + toolchain binary.
- `lib/std/` — standard-library Lex sources (all pass `lex check`).
- `templates/` — `lex new` starters (`default`, `api`, `service`).
- `examples/` — runnable samples (`lex run examples/hello.lex`).
- `docs/` — CLI reference, stdlib reference, embedding contract.
- `scripts/` — `activate.ps1` / `activate.sh` put `bin/` on PATH.

Quick start (PowerShell): `. scripts/activate.ps1; lex version`
Quick start (sh): `. scripts/activate.sh && lex version`

Flavours: `bin/lex` is either the full build (`lex gui` works) or the
~3.5 MB mini runtime (`lex sdk info` reports which).
"#;

const LICENSE_MIT: &str = r#"MIT License — LexiconLang SDK

Copyright (c) LexiconLang Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

const ACTIVATE_PS1: &str = r#"# Lexicon SDK activator (PowerShell). Usage: . scripts/activate.ps1
$SdkBin = Join-Path $PSScriptRoot ".." "bin" | Resolve-Path | Select-Object -ExpandProperty Path
if (($env:PATH -split ";") -notcontains $SdkBin) { $env:PATH = "$SdkBin;$env:PATH" }
Write-Host "Lexicon SDK active ($SdkBin)" -ForegroundColor Green
lex version
"#;

const ACTIVATE_SH: &str = r#"# Lexicon SDK activator (sh/bash). Usage: . scripts/activate.sh
SDK_BIN="$(cd "$(dirname "$0")/../bin" && pwd)"
case ":$PATH:" in
  *":$SDK_BIN:"*) ;;
  *) export PATH="$SDK_BIN:$PATH" ;;
esac
echo "Lexicon SDK active ($SDK_BIN)"
lex version
"#;

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Assemble the SDK tree. Copies the running binary into `bin/`.
pub fn export(out: Option<String>) -> Result<()> {
    export_to(&sdk_root(&out), true)
}

/// Same as [`export`], with silent mode for first-run auto-install:
/// `verbose=false` prints nothing (the installer's silence contract).
pub fn export_to(root: &Path, verbose: bool) -> Result<()> {
    let mut written: Vec<(String, u64)> = Vec::new();

    // Toolchain binary (the running exe — build the flavour to ship first).
    let exe = std::env::current_exe().context("locating running lex binary")?;
    let dest = root.join("bin").join(EXE_NAME);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if dest != exe {
        fs::copy(&exe, &dest)
            .with_context(|| format!("copying {} -> {}", exe.display(), dest.display()))?;
        #[cfg(unix)]
        crate::installer::make_executable(&dest)?;
    }
    let bin_size = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    if verbose {
        println!("bin/{} ({} bytes)", EXE_NAME, bin_size);
    }

    // Standard library.
    for (name, src) in [
        ("prelude.lex", STD_PRELUDE),
        ("collections.lex", STD_COLLECTIONS),
        ("json.lex", STD_JSON),
        ("http.lex", STD_HTTP),
        ("testing.lex", STD_TESTING),
        ("strings.lex", STD_STRINGS),
        ("strconv.lex", STD_STRCONV),
        ("math.lex", STD_MATH),
        ("sort.lex", STD_SORT),
        ("slices.lex", STD_SLICES),
        ("errors.lex", STD_ERRORS),
        ("path.lex", STD_PATH),
        ("fmt.lex", STD_FMT),
        ("time.lex", STD_TIME),
        ("bytes.lex", STD_BYTES),
        ("io.lex", STD_IO),
        ("bufio.lex", STD_BUFIO),
        ("maps.lex", STD_MAPS),
        ("cmp.lex", STD_CMP),
        ("iter.lex", STD_ITER),
        ("unicode/utf8.lex", STD_UTF8),
        ("container/list.lex", STD_LIST),
        ("container/heap.lex", STD_HEAP),
        ("container/ring.lex", STD_RING),
        ("os.lex", STD_OS),
        ("sync.lex", STD_SYNC),
        ("context.lex", STD_CONTEXT),
        ("log.lex", STD_LOG),
        ("path/filepath.lex", STD_FILEPATH),
        ("encoding/json.lex", STD_ENCODING_JSON),
        ("encoding/base64.lex", STD_BASE64),
        ("encoding/hex.lex", STD_HEX),
        ("encoding/csv.lex", STD_CSV),
        ("html.lex", STD_HTML),
        ("regexp.lex", STD_REGEXP),
        ("hash/fnv.lex", STD_FNV),
        ("hash/crc32.lex", STD_CRC32),
        ("hash/adler32.lex", STD_ADLER32),
        ("rand.lex", STD_RAND),
        ("crypto/subtle.lex", STD_SUBTLE),
        ("os/exec.lex", STD_EXEC),
        ("sync/atomic.lex", STD_ATOMIC),
        ("net/url.lex", STD_URL),
        ("slog.lex", STD_SLOG),
        ("flag.lex", STD_FLAG),
        ("mime.lex", STD_MIME),
        ("encoding/base32.lex", STD_BASE32),
        ("math/bits.lex", STD_BITS),
        ("crypto/sha256.lex", STD_SHA256),
        ("crypto/hmac.lex", STD_HMAC),
        ("unsafe.lex", STD_UNSAFE),
        ("runtime.lex", STD_RUNTIME),
        ("net/mail.lex", STD_MAIL),
        ("net/textproto.lex", STD_TEXTPROTO),
        ("mime/multipart.lex", STD_MULTIPART),
        ("encoding/pem.lex", STD_PEM),
        ("encoding/ascii85.lex", STD_ASCII85),
        ("image/color.lex", STD_COLOR),
        ("archive/tar.lex", STD_TAR),
        ("hash/crc64.lex", STD_CRC64),
        ("hash/maphash.lex", STD_MAPHASH),
        ("os/signal.lex", STD_SIGNAL),
        ("os/user.lex", STD_USER),
        ("expvar.lex", STD_EXPVAR),
        ("unicode/utf16.lex", STD_UTF16),
        ("native/console.lex", STD_NATIVE_CONSOLE),
        ("native/json.lex", STD_NATIVE_JSON),
        ("native/env.lex", STD_NATIVE_ENV),
        ("native/http.lex", STD_NATIVE_HTTP),
        ("native/time.lex", STD_NATIVE_TIME),
        ("native/file.lex", STD_NATIVE_FILE),
        ("native/process.lex", STD_NATIVE_PROCESS),
        ("native/text.lex", STD_NATIVE_TEXT),
        ("native/math.lex", STD_NATIVE_MATH),
        ("native/list.lex", STD_NATIVE_LIST),
        ("native/hash.lex", STD_NATIVE_HASH),
        ("native/rand.lex", STD_NATIVE_RAND),
        ("native/atomic.lex", STD_NATIVE_ATOMIC),
        ("native/sys.lex", STD_NATIVE_SYS),
    ] {
        write_file(&root, &format!("lib/std/{}", name), src, &mut written)?;
    }

    // Templates (mirror `lex new`).
    for (tpl, main) in [
        ("default", TPL_DEFAULT_MAIN),
        ("api", TPL_API_MAIN),
        ("service", TPL_SERVICE_MAIN),
    ] {
        write_file(&root, &format!("templates/{}/src/main.lex", tpl), main, &mut written)?;
        let mf = manifest(&format!("my-{}-app", tpl), tpl);
        write_file(&root, &format!("templates/{}/lexicon.toml", tpl), &mf, &mut written)?;
    }

    // Examples.
    write_file(&root, "examples/hello.lex", EX_HELLO, &mut written)?;
    write_file(&root, "examples/api.lex", EX_API, &mut written)?;

    // Docs.
    write_file(&root, "docs/CLI.md", DOC_CLI, &mut written)?;
    write_file(&root, "docs/STDLIB.md", DOC_STDLIB, &mut written)?;
    write_file(&root, "docs/EMBEDDING.md", DOC_EMBEDDING, &mut written)?;

    // Meta + scripts.
    write_file(&root, "README.md", SDK_README, &mut written)?;
    write_file(&root, "VERSION", &format!("{}\n", SDK_VERSION), &mut written)?;
    write_file(&root, "LICENSE-MIT.md", LICENSE_MIT, &mut written)?;
    write_file(&root, "scripts/activate.ps1", ACTIVATE_PS1, &mut written)?;
    write_file(&root, "scripts/activate.sh", ACTIVATE_SH, &mut written)?;

    let total: u64 = written.iter().map(|(_, n)| n).sum::<u64>() + bin_size;
    if verbose {
        println!(
            "SDK exported to {} ({} files + binary, {} bytes total, gui={})",
            root.display(),
            written.len(),
            total,
            HAS_GUI
        );
    }
    Ok(())
}

/// Installed-SDK version probe used by first-run auto-install: `None`
/// when missing/unreadable (→ install), `Some(v)` otherwise (→ compare
/// with [`SDK_VERSION`], update on mismatch).
pub fn installed_version(root: &Path) -> Option<String> {
    fs::read_to_string(root.join("VERSION"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

const REQUIRED: &[&str] = &[
    "VERSION",
    "README.md",
    "LICENSE-MIT.md",
    "docs/CLI.md",
    "docs/STDLIB.md",
    "docs/EMBEDDING.md",
    "lib/std/prelude.lex",
    "lib/std/collections.lex",
    "lib/std/json.lex",
    "lib/std/http.lex",
    "lib/std/testing.lex",
    "lib/std/strings.lex",
    "lib/std/strconv.lex",
    "lib/std/math.lex",
    "lib/std/sort.lex",
    "lib/std/slices.lex",
    "lib/std/errors.lex",
    "lib/std/path.lex",
    "lib/std/fmt.lex",
    "lib/std/time.lex",
    "lib/std/bytes.lex",
    "lib/std/io.lex",
    "lib/std/bufio.lex",
    "lib/std/maps.lex",
    "lib/std/cmp.lex",
    "lib/std/iter.lex",
    "lib/std/unicode/utf8.lex",
    "lib/std/container/list.lex",
    "lib/std/container/heap.lex",
    "lib/std/container/ring.lex",
    "lib/std/os.lex",
    "lib/std/sync.lex",
    "lib/std/context.lex",
    "lib/std/log.lex",
    "lib/std/path/filepath.lex",
    "lib/std/encoding/json.lex",
    "lib/std/encoding/base64.lex",
    "lib/std/encoding/hex.lex",
    "lib/std/encoding/csv.lex",
    "lib/std/html.lex",
    "lib/std/regexp.lex",
    "lib/std/hash/fnv.lex",
    "lib/std/hash/crc32.lex",
    "lib/std/hash/adler32.lex",
    "lib/std/rand.lex",
    "lib/std/crypto/subtle.lex",
    "lib/std/os/exec.lex",
    "lib/std/sync/atomic.lex",
    "lib/std/net/url.lex",
    "lib/std/slog.lex",
    "lib/std/flag.lex",
    "lib/std/mime.lex",
    "lib/std/encoding/base32.lex",
    "lib/std/math/bits.lex",
    "lib/std/crypto/sha256.lex",
    "lib/std/crypto/hmac.lex",
    "lib/std/unsafe.lex",
    "lib/std/runtime.lex",
    "lib/std/net/mail.lex",
    "lib/std/net/textproto.lex",
    "lib/std/mime/multipart.lex",
    "lib/std/encoding/pem.lex",
    "lib/std/encoding/ascii85.lex",
    "lib/std/image/color.lex",
    "lib/std/archive/tar.lex",
    "lib/std/hash/crc64.lex",
    "lib/std/hash/maphash.lex",
    "lib/std/os/signal.lex",
    "lib/std/os/user.lex",
    "lib/std/expvar.lex",
    "lib/std/unicode/utf16.lex",
    "lib/std/native/console.lex",
    "lib/std/native/json.lex",
    "lib/std/native/env.lex",
    "lib/std/native/http.lex",
    "lib/std/native/time.lex",
    "lib/std/native/file.lex",
    "lib/std/native/process.lex",
    "lib/std/native/text.lex",
    "lib/std/native/math.lex",
    "lib/std/native/list.lex",
    "lib/std/native/hash.lex",
    "lib/std/native/rand.lex",
    "lib/std/native/atomic.lex",
    "lib/std/native/sys.lex",
    "templates/default/src/main.lex",
    "templates/default/lexicon.toml",
    "templates/api/src/main.lex",
    "templates/api/lexicon.toml",
    "templates/service/src/main.lex",
    "templates/service/lexicon.toml",
    "examples/hello.lex",
    "examples/api.lex",
    "scripts/activate.ps1",
    "scripts/activate.sh",
];

/// Binary budgets enforced by `verify` (Onda 0.5, see Docs/FEATURE_MATRIX.md).
/// Mini (no-default) must stay tiny; full must not accidentally ship debug-size.
const BUDGET_MINI_BYTES: u64 = 6 * 1024 * 1024;
const BUDGET_FULL_BYTES: u64 = 16 * 1024 * 1024;

/// Verify a previously exported tree: files present + `bin/lex version` runs
/// + binary size within budget.
pub fn verify(path: Option<String>) -> Result<()> {
    let root = sdk_root(&path);
    let mut missing = 0;
    for rel in REQUIRED {
        if !root.join(rel).is_file() {
            println!("MISSING {}", rel);
            missing += 1;
        }
    }
    let bin = root.join("bin").join(EXE_NAME);
    let bin_ok = bin.is_file();
    if !bin_ok {
        println!("MISSING bin/{}", EXE_NAME);
        missing += 1;
    }
    let bin_size: u64 = bin.metadata().map(|m| m.len()).unwrap_or(0);
    // Flavour heuristic: mini builds stay well under the mini budget.
    let (flavour, budget) = if bin_size <= BUDGET_MINI_BYTES {
        ("mini", BUDGET_MINI_BYTES)
    } else {
        ("full", BUDGET_FULL_BYTES)
    };
    let mut budget_ok = true;
    if bin_ok {
        if bin_size <= budget {
            println!(
                "bin/{} {} bytes ({} budget {} bytes: OK)",
                EXE_NAME, bin_size, flavour, budget
            );
        } else {
            println!(
                "bin/{} {} bytes EXCEEDS {} budget {} bytes",
                EXE_NAME, bin_size, flavour, budget
            );
            budget_ok = false;
        }
    }
    let mut run_ok = false;
    if bin_ok {
        match std::process::Command::new(&bin).arg("version").output() {
            Ok(o) if o.status.success() => {
                let first = String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                println!("bin/{} runs: {}", EXE_NAME, first.trim());
                run_ok = first.contains("lex");
            }
            Ok(o) => println!("bin/{} exited with {}", EXE_NAME, o.status),
            Err(e) => println!("bin/{} failed to start: {}", EXE_NAME, e),
        }
    }
    if missing == 0 && run_ok && budget_ok {
        println!("SDK verify OK at {} ({} files)", root.display(), REQUIRED.len() + 1);
        Ok(())
    } else {
        let problems =
            missing + if run_ok { 0 } else { 1 } + if budget_ok { 0 } else { 1 };
        anyhow::bail!("SDK verify FAILED at {} ({} problem(s))", root.display(), problems);
    }
}

/// Print SDK/toolchain provenance.
pub fn info() -> Result<()> {
    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
    let size = std::env::current_exe()
        .and_then(|p| std::fs::metadata(p).map(|m| m.len()))
        .unwrap_or(0);
    println!("Lexicon SDK {}", SDK_VERSION);
    println!("flavour: {}", if HAS_GUI { "full (+gui)" } else { "mini (no gui)" });
    println!("binary: {} ({} bytes)", exe, size);
    println!("default export dir: {}", sdk_root(&None).display());
    println!("contents: bin lib/std templates examples docs scripts");
    Ok(())
}

#[cfg(test)]
mod sdk_tests {
    use super::*;

    #[test]
    fn missing_sdk_dir_reports_none() {
        let dir = std::env::temp_dir().join("lex-sdk-probe-missing-dir-xyz");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(installed_version(&dir), None);
    }

    #[test]
    fn version_file_round_trips_trimmed() {
        let dir = std::env::temp_dir().join("lex-sdk-probe-version");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("VERSION"), "0.3.0\n").unwrap();
        assert_eq!(installed_version(&dir).as_deref(), Some("0.3.0"));
        std::fs::write(dir.join("VERSION"), "\n  \n").unwrap();
        assert_eq!(installed_version(&dir), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
