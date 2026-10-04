# Lexicon Snippets Catalog

> 94 snippets. Gerado automaticamente de `snippets/lexicon.json` — 
> a fonte de verdade é o JSON; este arquivo é espelho.

| Prefix | Snippet | Description |
|---|---|---|
| `as` | As Cast | Explicit cast (required for narrowing) |
| `assert` | Assert | Assertion |
| `afn` | Async Function | Async function declaration |
| `/*` | Block Comment | Block comment |
| `break` | Break | Break statement |
| `cfg` | Cfg Feature | Conditional compilation attribute |
| `chan` | Channel | Create channel |
| `chanr` | Channel Receive | Receive from a Channel (None when closed and drained) |
| `chans` | Channel Send | Send on a Channel (fails on closed/full) |
| `class` | Class | Class definition |
| `classe` | Class Extends | Class with inheritance |
| `classi` | Class Implements | Class implementing a trait |
| `clos` | Closure | Closure/lambda |
| `closb` | Closure Block | Closure with block body |
| `const` | Const | Compile-time constant |
| `cont` | Continue | Continue statement |
| `dbcon` | Db Connect | Open database connection |
| `dbx` | Db Execute | Run DDL/DML statement |
| `dbi` | Db Insert | Parameterized INSERT |
| `dbq` | Db Query | Run SELECT query |
| `dnscheck` | DNS Check | Resolve a host (Node dns.lookup parity) |
| `defer` | Defer | Deferred call (runs at scope unwind) |
| `letd` | Destructuring Let | Tuple destructuring |
| `dowhile` | Do While | Do-while loop |
| `///` | Doc Comment | Doc comment (picked up by lex doc) |
| `enum` | Enum | Enum definition |
| `envg` | Env Get | Read environment variable |
| `extern` | Extern Block | Foreign function declaration |
| `for` | For Loop | For loop |
| `forr` | For Range | For over range |
| `api` | Full API File | Minimal runnable HTTP API file |
| `fn` | Function | Function declaration |
| `bgfn` | Generic Bounded Function | Generic function with trait bound |
| `gfn` | Generic Function | Generic function |
| `httpg` | HTTP Client Get | HTTP GET client request |
| `httpp` | HTTP Client Post | HTTP POST client request |
| `del` | HTTP Delete Route | HTTP DELETE route handler |
| `get` | HTTP Get Route | HTTP GET route handler |
| `post` | HTTP Post Route | HTTP POST route handler |
| `put` | HTTP Put Route | HTTP PUT route handler |
| `serve` | HTTP Serve | Start HTTP server (literal bind address) |
| `ife` | If Else | If-else statement |
| `if` | If Statement | If statement |
| `impa` | Import Alias | Import with alias |
| `impc` | Import Core | Import core module |
| `imp` | Import Module | Import module with :: path |
| `loop` | Infinite Loop | Infinite loop |
| `listnew` | List New | Create a List and push an item |
| `iface` | Interface | Interface definition |
| `jsonp` | Json Parse | Parse JSON string |
| `jsons` | Json Stringify | Value to JSON string |
| `let` | Let Variable | Immutable variable |
| `//` | Line Comment | Line comment |
| `main` | Main Function | Main entry point |
| `match` | Match | Match expression |
| `mbool` | Match Bool Exhaustive | Exhaustive bool match |
| `matchg` | Match Guard | Match with guard clause |
| `mopt` | Match Option | Match Option |
| `mres` | Match Result | Match Result |
| `mod` | Module Declaration | Module declaration |
| `mapnew` | Map New | Create a Map and set a key/value pair |
| `mut` | Mutable Variable | Mutable variable |
| `none` | Option None | Option None |
| `some` | Option Some | Option Some |
| `panic` | Panic | Fatal runtime error (Never type) |
| `pipe` | Pipe | Pipe operator |
| `print` | Print | Print to stdout |
| `printf` | Print Concat | Print with concatenation |
| `printenv` | Print Env | Read env var into variable, then print (evaluates for real) |
| `println` | Println | Print line to stdout |
| `pfn` | Public Function | Public function declaration |
| `pstruct` | Public Struct | Public struct definition |
| `err` | Result Err | Result Err |
| `ok` | Result Ok | Result Ok |
| `optunw` | Option Unwrap Or | Unwrap Option or return the default |
| `ret` | Return | Return with value |
| `retv` | Return Void | Bare return |
| `spawn` | Spawn Task | Spawn lightweight task |
| `stat` | Static | Static variable |
| `struct` | Struct | Struct definition |
| `slit` | Struct Literal | Struct literal |
| `switch` | Switch | Switch statement with cases and default |
| `test` | Test Function | Test picked up by lex test |
| `testnet` | Net Smoke Test | Smoke-test TCP/HTTP/UDP/URL/WS (CLI + Http::get) |
| `tlscheck` | TLS Check | Validate TLS config for a host (Node tls parity) |
| `todo` | Todo Comment | TODO marker |
| `trait` | Trait | Trait definition |
| `try` | Try Catch | Try-catch block |
| `type` | Type Alias | Type alias |
| `letc` | Typed Let | Typed variable declaration |
| `unsafe` | Unsafe Block | Unsafe block (raw ops confined here) |
| `vfn` | Variadic Function | Variadic function (trailing ... must be last) |
| `where` | Where Clause Function | Function with where clause |
| `while` | While Loop | While loop |

## Example expansions

### `api` — minimal runnable HTTP API
```lex
import core::net::Http;

let app_env = Env::get("APP_ENV");

@Get("${1:/hello}")
pub fn ${2:hello}() -> String {
	return ${3:"Hello from Lexicon!"};
}

pub fn main() -> void {
	print("env=" + app_env);
	Http::serve("${4:0.0.0.0:3000}");
}
```

### `serve` + `get` — route pair
```lex
@Get("${1:/path}")
pub fn ${2:handler}() -> String {
	return ${3:"response"};$0
}
Http::serve("${1:0.0.0.0:3000}");
```
