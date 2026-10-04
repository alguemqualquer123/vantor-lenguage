//! Autocomplete engine (`lex complete`, REPL `:complete`, `lex lsp`).
//!
//! Single source of truth for the callable surface of LexLang. Every entry
//! here mirrors something the toolchain really honours:
//! - Native `Module::member` builtins (Console, Json, Env, Http, Time, File,
//!   Process, Text, Math, List, Hash, Rand, Atomic) → executed by the real
//!   interpreter in `interp.rs::call_module` (no import needed).
//! - `std::<package>` file modules (`strings::ToUpper`, …) → loaded from
//!   `lib/std/**/*.lex` by the module loader; `import std::<path>;` first.
//! - `@Get/@Post/@Put/@Delete` → route extraction for the real axum server.
//! - `@Test` → `lex test` discovery.

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Module,
    Function,
    Decorator,
    Keyword,
    Snippet,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Module => "module",
            Kind::Function => "function",
            Kind::Decorator => "decorator",
            Kind::Keyword => "keyword",
            Kind::Snippet => "snippet",
        }
    }

    /// LSP `CompletionItemKind` number.
    pub fn lsp_kind(&self) -> u32 {
        match self {
            Kind::Module => 9,
            Kind::Function => 3,
            Kind::Decorator => 24, // Operator-ish; VSCode renders distinctly
            Kind::Keyword => 14,
            Kind::Snippet => 15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub label: &'static str,
    pub kind: Kind,
    pub detail: &'static str,
    pub doc: &'static str,
    pub insert: &'static str,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct Module {
    pub name: &'static str,
    pub doc: &'static str,
    pub members: &'static [Item],
}

const HTTP_MEMBERS: &[Item] = &[
    Item { label: "serve", kind: Kind::Function, detail: "Http::serve(addr: String) -> void", doc: "Starts the REAL HTTP server (axum) with CORS, JSON envelopes, 404/405 and request log. Blocks until killed — ideal for `lex run --watch`.", insert: "serve(\"0.0.0.0:3000\")" },
    Item { label: "get", kind: Kind::Function, detail: "Http::get(url: String) -> String", doc: "Real blocking HTTP fetch (reqwest) executed at runtime; returns the response body. Also `Http.get(…)` (PUT/POST bodies pass as 2nd arg).", insert: "get(\"https://…\")" },
    Item { label: "post", kind: Kind::Function, detail: "Http::post(url: String, body: String) -> String", doc: "Real blocking HTTP POST; returns the response body.", insert: "post(\"https://…\", body)" },
    Item { label: "put", kind: Kind::Function, detail: "Http::put(url: String, body: String) -> String", doc: "Real blocking HTTP PUT; returns the response body.", insert: "put(\"https://…\", body)" },
    Item { label: "delete", kind: Kind::Function, detail: "Http::delete(url: String) -> String", doc: "Real blocking HTTP DELETE; returns the response body.", insert: "delete(\"https://…\")" },
    Item { label: "head", kind: Kind::Function, detail: "Http::head(url: String) -> String", doc: "Real blocking HTTP HEAD.", insert: "head(\"https://…\")" },
];

const JSON_MEMBERS: &[Item] = &[
    Item { label: "parse", kind: Kind::Function, detail: "Json::parse(text: String) -> value", doc: "Parses JSON at runtime (objects → struct, arrays → list). Aborts on invalid input — guard with `Json::valid`. Also `Json.parse(…)`.", insert: "parse(\"{…}\")" },
    Item { label: "serialize", kind: Kind::Function, detail: "Json::serialize(value) -> String", doc: "Serializes a value to JSON (also `Json::stringify`, same builtin).", insert: "serialize(value)" },
    Item { label: "stringify", kind: Kind::Function, detail: "Json::stringify(value) -> String", doc: "Alias of `Json::serialize` (same native builtin).", insert: "stringify(value)" },
    Item { label: "valid", kind: Kind::Function, detail: "Json::valid(text: String) -> bool", doc: "Reports whether the text is well-formed JSON (never aborts).", insert: "valid(text)" },
];

const CONSOLE_MEMBERS: &[Item] = &[
    Item { label: "writeLine", kind: Kind::Function, detail: "Console::writeLine(text: String) -> void", doc: "Prints a line to stdout (also `Console.writeLine(…)`). Supports Quake-style colors `^0`–`^8` inside the string, `^9` = reset, `^^` = literal `^` (reset auto-appended; `NO_COLOR` disables).", insert: "writeLine(\"…\")" },
    Item { label: "write", kind: Kind::Function, detail: "Console::write(text: String) -> void", doc: "Prints without trailing newline (also `Console.write(…)`).", insert: "write(\"…\")" },
    Item { label: "log", kind: Kind::Function, detail: "Console::log(text: String) -> void", doc: "Log-level line print (also `Console.log(…)`).", insert: "log(\"…\")" },
];

const ENV_MEMBERS: &[Item] = &[
    Item { label: "get", kind: Kind::Function, detail: "Env::get(name: String) -> String", doc: "Reads an environment variable (`\"NOT_FOUND\"` when unset — prefer `std::os::Getenv` for `\"\"`).", insert: "get(\"VAR\")" },
    Item { label: "set", kind: Kind::Function, detail: "Env::set(name: String, value: String) -> String", doc: "Sets an environment variable; returns the value.", insert: "set(\"VAR\", \"v\")" },
    Item { label: "exists", kind: Kind::Function, detail: "Env::exists(name: String) -> bool", doc: "Reports whether the variable is set.", insert: "exists(\"VAR\")" },
];

const TIME_MEMBERS: &[Item] = &[
    Item { label: "now_unix_ms", kind: Kind::Function, detail: "Time::now_unix_ms() -> i64", doc: "Wall-clock milliseconds since the Unix epoch (backs `std::time`).", insert: "now_unix_ms()" },
    Item { label: "now_unix_s", kind: Kind::Function, detail: "Time::now_unix_s() -> i64", doc: "Wall-clock seconds since the Unix epoch.", insert: "now_unix_s()" },
    Item { label: "sleep", kind: Kind::Function, detail: "Time::sleep(ms: i64) -> void", doc: "Suspends execution for `ms` milliseconds.", insert: "sleep(100)" },
];

const FILE_MEMBERS: &[Item] = &[
    Item { label: "read_string", kind: Kind::Function, detail: "File::read_string(path: String) -> String", doc: "Reads a whole file (aborts when unreadable — check `File::exists` first). Backs `std::os`.", insert: "read_string(path)" },
    Item { label: "write_string", kind: Kind::Function, detail: "File::write_string(path: String, data: String) -> bool", doc: "Writes a whole file; reports success.", insert: "write_string(path, data)" },
    Item { label: "exists", kind: Kind::Function, detail: "File::exists(path: String) -> bool", doc: "Reports whether the path exists.", insert: "exists(path)" },
    Item { label: "is_dir", kind: Kind::Function, detail: "File::is_dir(path: String) -> bool", doc: "Reports whether the path is a directory.", insert: "is_dir(path)" },
    Item { label: "is_file", kind: Kind::Function, detail: "File::is_file(path: String) -> bool", doc: "Reports whether the path is a file.", insert: "is_file(path)" },
    Item { label: "remove", kind: Kind::Function, detail: "File::remove(path: String) -> bool", doc: "Removes a file.", insert: "remove(path)" },
    Item { label: "remove_dir", kind: Kind::Function, detail: "File::remove_dir(path: String) -> bool", doc: "Removes an (empty) directory.", insert: "remove_dir(path)" },
    Item { label: "mkdir", kind: Kind::Function, detail: "File::mkdir(path: String) -> bool", doc: "Creates one directory level.", insert: "mkdir(path)" },
    Item { label: "mkdir_all", kind: Kind::Function, detail: "File::mkdir_all(path: String) -> bool", doc: "Creates a directory and parents.", insert: "mkdir_all(path)" },
    Item { label: "read_dir", kind: Kind::Function, detail: "File::read_dir(path: String) -> [String]", doc: "Lists entry names of a directory.", insert: "read_dir(path)" },
    Item { label: "size", kind: Kind::Function, detail: "File::size(path: String) -> i64", doc: "Byte size, or -1 when missing.", insert: "size(path)" },
    Item { label: "getwd", kind: Kind::Function, detail: "File::getwd() -> String", doc: "Current working directory.", insert: "getwd()" },
    Item { label: "temp_dir", kind: Kind::Function, detail: "File::temp_dir() -> String", doc: "OS temporary directory.", insert: "temp_dir()" },
    Item { label: "home_dir", kind: Kind::Function, detail: "File::home_dir() -> String", doc: "Current user home (`USERPROFILE`/`HOME`).", insert: "home_dir()" },
];

const PROCESS_MEMBERS: &[Item] = &[
    Item { label: "args", kind: Kind::Function, detail: "Process::args() -> [String]", doc: "Command-line arguments (backs `std::flag::Parse`).", insert: "args()" },
    Item { label: "exit", kind: Kind::Function, detail: "Process::exit(code: i64) -> void", doc: "Terminates the process with `code`.", insert: "exit(1)" },
    Item { label: "hostname", kind: Kind::Function, detail: "Process::hostname() -> String", doc: "Machine hostname (`COMPUTERNAME`/`HOSTNAME`).", insert: "hostname()" },
    Item { label: "output", kind: Kind::Function, detail: "Process::output(prog: String, argv: [String], input: String) -> Output", doc: "Spawns a child and captures it; returns `Output { code, stdout, stderr }` (spawn failure → `code == -1`). Backs `std::os::exec`.", insert: "output(\"${1:prog}\", [${2:args}], \"\")" },
];

const TEXT_MEMBERS: &[Item] = &[
    Item { label: "code_at", kind: Kind::Function, detail: "Text::code_at(s: String, i: i64) -> i64", doc: "Code point at char offset `i` (-1 when out of range).", insert: "code_at(s, ${1:i})" },
    Item { label: "from_code", kind: Kind::Function, detail: "Text::from_code(cp: i64) -> String", doc: "One-character string for a code point.", insert: "from_code(${1:65})" },
    Item { label: "len", kind: Kind::Function, detail: "Text::len(s: String) -> i64", doc: "Character (not byte) count.", insert: "len(s)" },
    Item { label: "slice", kind: Kind::Function, detail: "Text::slice(s: String, lo: i64, hi: i64) -> String", doc: "Char-offset `[lo:hi)` slice, O(n), clamped.", insert: "slice(s, ${1:lo}, ${2:hi})" },
    Item { label: "index_of", kind: Kind::Function, detail: "Text::index_of(s: String, sub: String) -> i64", doc: "First char offset of `sub`, or -1.", insert: "index_of(s, sub)" },
    Item { label: "last_index_of", kind: Kind::Function, detail: "Text::last_index_of(s: String, sub: String) -> i64", doc: "Last char offset of `sub`, or -1.", insert: "last_index_of(s, sub)" },
    Item { label: "starts_with", kind: Kind::Function, detail: "Text::starts_with(s: String, prefix: String) -> bool", doc: "Prefix test.", insert: "starts_with(s, prefix)" },
    Item { label: "ends_with", kind: Kind::Function, detail: "Text::ends_with(s: String, suffix: String) -> bool", doc: "Suffix test.", insert: "ends_with(s, suffix)" },
    Item { label: "repeat", kind: Kind::Function, detail: "Text::repeat(s: String, n: i64) -> String", doc: "`n` copies concatenated.", insert: "repeat(s, ${1:n})" },
    Item { label: "join", kind: Kind::Function, detail: "Text::join(list, sep: String) -> String", doc: "Joins list elements with `sep`.", insert: "join(list, sep)" },
];

const MATH_MEMBERS: &[Item] = &[
    Item { label: "sin", kind: Kind::Function, detail: "Math::sin(x: f64) -> f64", doc: "Native sine (radians).", insert: "sin(${1:x})" },
    Item { label: "cos", kind: Kind::Function, detail: "Math::cos(x: f64) -> f64", doc: "Native cosine.", insert: "cos(${1:x})" },
    Item { label: "tan", kind: Kind::Function, detail: "Math::tan(x: f64) -> f64", doc: "Native tangent.", insert: "tan(${1:x})" },
    Item { label: "asin", kind: Kind::Function, detail: "Math::asin(x: f64) -> f64", doc: "Native arcsine.", insert: "asin(${1:x})" },
    Item { label: "acos", kind: Kind::Function, detail: "Math::acos(x: f64) -> f64", doc: "Native arccosine.", insert: "acos(${1:x})" },
    Item { label: "atan", kind: Kind::Function, detail: "Math::atan(x: f64) -> f64", doc: "Native arctangent.", insert: "atan(${1:x})" },
    Item { label: "atan2", kind: Kind::Function, detail: "Math::atan2(y: f64, x: f64) -> f64", doc: "Quadrant-aware arctangent.", insert: "atan2(${1:y}, ${2:x})" },
    Item { label: "sqrt", kind: Kind::Function, detail: "Math::sqrt(x: f64) -> f64", doc: "Native square root.", insert: "sqrt(${1:x})" },
    Item { label: "cbrt", kind: Kind::Function, detail: "Math::cbrt(x: f64) -> f64", doc: "Native cube root.", insert: "cbrt(${1:x})" },
    Item { label: "exp", kind: Kind::Function, detail: "Math::exp(x: f64) -> f64", doc: "Native e^x.", insert: "exp(${1:x})" },
    Item { label: "ln", kind: Kind::Function, detail: "Math::ln(x: f64) -> f64", doc: "Native natural logarithm.", insert: "ln(${1:x})" },
    Item { label: "log", kind: Kind::Function, detail: "Math::log(x: f64, base: f64) -> f64", doc: "Logarithm in `base`.", insert: "log(${1:x}, ${2:base})" },
    Item { label: "log10", kind: Kind::Function, detail: "Math::log10(x: f64) -> f64", doc: "Base-10 logarithm.", insert: "log10(${1:x})" },
    Item { label: "log2", kind: Kind::Function, detail: "Math::log2(x: f64) -> f64", doc: "Base-2 logarithm.", insert: "log2(${1:x})" },
    Item { label: "pow", kind: Kind::Function, detail: "Math::pow(x: f64, y: f64) -> f64", doc: "x raised to y.", insert: "pow(${1:x}, ${2:y})" },
    Item { label: "hypot", kind: Kind::Function, detail: "Math::hypot(x: f64, y: f64) -> f64", doc: "Overflow-safe sqrt(x²+y²).", insert: "hypot(${1:x}, ${2:y})" },
    Item { label: "floor", kind: Kind::Function, detail: "Math::floor(x: f64) -> f64", doc: "Round down.", insert: "floor(${1:x})" },
    Item { label: "ceil", kind: Kind::Function, detail: "Math::ceil(x: f64) -> f64", doc: "Round up.", insert: "ceil(${1:x})" },
    Item { label: "round", kind: Kind::Function, detail: "Math::round(x: f64) -> f64", doc: "Half away from zero.", insert: "round(${1:x})" },
    Item { label: "trunc", kind: Kind::Function, detail: "Math::trunc(x: f64) -> f64", doc: "Truncate toward zero.", insert: "trunc(${1:x})" },
    Item { label: "abs", kind: Kind::Function, detail: "Math::abs(x: f64) -> f64", doc: "Absolute value.", insert: "abs(${1:x})" },
    Item { label: "sign", kind: Kind::Function, detail: "Math::sign(x: f64) -> f64", doc: "-1, 0 or +1.", insert: "sign(${1:x})" },
    Item { label: "fmod", kind: Kind::Function, detail: "Math::fmod(x: f64, y: f64) -> f64", doc: "Floating-point remainder.", insert: "fmod(${1:x}, ${2:y})" },
    Item { label: "max", kind: Kind::Function, detail: "Math::max(a, b)", doc: "Larger operand (int-preserving for ints).", insert: "max(${1:a}, ${2:b})" },
    Item { label: "min", kind: Kind::Function, detail: "Math::min(a, b)", doc: "Smaller operand (int-preserving for ints).", insert: "min(${1:a}, ${2:b})" },
];

const LIST_MEMBERS: &[Item] = &[
    Item { label: "sort_ints", kind: Kind::Function, detail: "List::sort_ints(xs) -> [i64]", doc: "Native ascending sort of ints (backs `std::sort::Ints`).", insert: "sort_ints(${1:xs})" },
    Item { label: "sort_floats", kind: Kind::Function, detail: "List::sort_floats(xs) -> [f64]", doc: "Native ascending sort of floats.", insert: "sort_floats(${1:xs})" },
    Item { label: "sort_strings", kind: Kind::Function, detail: "List::sort_strings(xs) -> [String]", doc: "Native ascending sort of strings.", insert: "sort_strings(${1:xs})" },
    Item { label: "reverse", kind: Kind::Function, detail: "List::reverse(xs) -> list", doc: "Reversed copy.", insert: "reverse(${1:xs})" },
];

const HASH_MEMBERS: &[Item] = &[
    Item { label: "fnv1a32", kind: Kind::Function, detail: "Hash::fnv1a32(s: String) -> i64", doc: "Native 32-bit FNV-1a.", insert: "fnv1a32(${1:s})" },
    Item { label: "fnv1a64", kind: Kind::Function, detail: "Hash::fnv1a64(s: String) -> i64", doc: "Native 64-bit FNV-1a (bits-wrapped into i64).", insert: "fnv1a64(${1:s})" },
    Item { label: "crc32", kind: Kind::Function, detail: "Hash::crc32(s: String) -> i64", doc: "Native IEEE CRC-32.", insert: "crc32(${1:s})" },
    Item { label: "adler32", kind: Kind::Function, detail: "Hash::adler32(s: String) -> i64", doc: "Native Adler-32.", insert: "adler32(${1:s})" },
    Item { label: "crc64", kind: Kind::Function, detail: "Hash::crc64(s: String) -> i64", doc: "Native ECMA CRC-64.", insert: "crc64(${1:s})" },
    Item { label: "maphash", kind: Kind::Function, detail: "Hash::maphash(seed: i64, s: String) -> i64", doc: "Native seeded FNV-1a64.", insert: "maphash(${1:seed}, ${2:s})" },
    Item { label: "sha256hex", kind: Kind::Function, detail: "Hash::sha256hex(s: String) -> String", doc: "Native SHA-256 hex (backs `sha256::Sum`).", insert: "sha256hex(${1:s})" },
];

const RAND_MEMBERS: &[Item] = &[
    Item { label: "intn", kind: Kind::Function, detail: "Rand::intn(n: i64) -> i64", doc: "Uniform value in [0, n).", insert: "intn(${1:n})" },
    Item { label: "int", kind: Kind::Function, detail: "Rand::int() -> i64", doc: "Random i64 (xorshift64*, wall-clock seeded).", insert: "int()" },
    Item { label: "float", kind: Kind::Function, detail: "Rand::float() -> f64", doc: "Uniform float in [0, 1).", insert: "float()" },
    Item { label: "bytes", kind: Kind::Function, detail: "Rand::bytes(n: i64) -> [i64]", doc: "`n` random bytes.", insert: "bytes(${1:n})" },
    Item { label: "seed", kind: Kind::Function, detail: "Rand::seed(s: i64) -> void", doc: "Reseeds the generator (deterministic replays).", insert: "seed(${1:s})" },
];

const ATOMIC_MEMBERS: &[Item] = &[
    Item { label: "make", kind: Kind::Function, detail: "Atomic::make(v: i64) -> id", doc: "Allocates a shared atomic-i64 cell; returns its handle (backs `std::sync::atomic`).", insert: "make(${1:v})" },
    Item { label: "get", kind: Kind::Function, detail: "Atomic::get(id: i64) -> i64", doc: "Atomic load.", insert: "get(${1:id})" },
    Item { label: "set", kind: Kind::Function, detail: "Atomic::set(id: i64, v: i64) -> void", doc: "Atomic store.", insert: "set(${1:id}, ${2:v})" },
    Item { label: "add", kind: Kind::Function, detail: "Atomic::add(id: i64, d: i64) -> i64", doc: "Atomically adds `d`; returns the new value.", insert: "add(${1:id}, ${2:d})" },
    Item { label: "swap", kind: Kind::Function, detail: "Atomic::swap(id: i64, v: i64) -> i64", doc: "Atomically swaps; returns the old value.", insert: "swap(${1:id}, ${2:v})" },
    Item { label: "cas", kind: Kind::Function, detail: "Atomic::cas(id: i64, old: i64, new: i64) -> bool", doc: "Atomic compare-and-swap.", insert: "cas(${1:id}, ${2:old}, ${3:new})" },
    Item { label: "drop", kind: Kind::Function, detail: "Atomic::drop(id: i64) -> void", doc: "Releases the cell.", insert: "drop(${1:id})" },
];

// ---------------------------------------------------------------------------
// File modules: `import std::<path>;` then `key::member(…)` (the key is the
// last path segment). Every member below is a real `pub fn` in
// `lib/std/**/*.lex`, executed by the module loader.
// ---------------------------------------------------------------------------

macro_rules! stdmod {
    ($name:literal, $doc:literal, [$($label:literal : $detail:literal : $docm:literal),* $(,)?]) => {
        &[$(Item { label: $label, kind: Kind::Function, detail: $detail, doc: $docm, insert: concat!($label, "($0)") }),*]
    };
}

const STD_STRINGS: &[Item] = stdmod!("strings", "Go-parity strings (`import std::strings;`).", [
    "ToUpper": "strings::ToUpper(s: String) -> String": "Upper-cases (native).",
    "ToLower": "strings::ToLower(s: String) -> String": "Lower-cases (native).",
    "Contains": "strings::Contains(s: String, sub: String) -> bool": "Substring test.",
    "HasPrefix": "strings::HasPrefix(s: String, p: String) -> bool": "Prefix test (native).",
    "HasSuffix": "strings::HasSuffix(s: String, sfx: String) -> bool": "Suffix test (native).",
    "Split": "strings::Split(s: String, sep: String) -> [String]": "Splits around `sep`.",
    "Join": "strings::Join(elems, sep: String) -> String": "Joins with `sep` (native).",
    "Repeat": "strings::Repeat(s: String, n: i64) -> String": "n copies (native).",
    "ReplaceAll": "strings::ReplaceAll(s: String, from: String, to: String) -> String": "Replaces every instance.",
    "TrimSpace": "strings::TrimSpace(s: String) -> String": "Trims ASCII whitespace.",
    "Trim": "strings::Trim(s: String, cutset: String) -> String": "Trims cutset both ends.",
    "Index": "strings::Index(s: String, sub: String) -> i64": "First index or -1 (native).",
    "LastIndex": "strings::LastIndex(s: String, sub: String) -> i64": "Last index or -1.",
    "Count": "strings::Count(s: String, sub: String) -> i64": "Counts instances.",
    "Fields": "strings::Fields(s: String) -> [String]": "Whitespace-separated fields.",
    "Cut": "strings::Cut(s: String, sep: String) -> [before, after, found]": "Cuts at first `sep`.",
    "Slice": "strings::Slice(s: String, lo: i64, hi: i64) -> String": "Char-offset slice (native).",
    "RuneCount": "strings::RuneCount(s: String) -> i64": "Character count.",
]);

const STD_STRCONV: &[Item] = stdmod!("strconv", "String conversions (`import std::strconv;`).", [
    "Itoa": "strconv::Itoa(i: i64) -> String": "Decimal rendering.",
    "Atoi": "strconv::Atoi(s: String) -> ParseIntResult": "`{ value, ok, err }`.",
    "FormatInt": "strconv::FormatInt(i: i64, base: i64) -> String": "Renders in base 2-36.",
    "ParseInt": "strconv::ParseInt(s: String, base: i64) -> ParseIntResult": "Parses with sign.",
    "ParseFloat": "strconv::ParseFloat(s: String) -> ParseFloatResult": "Parses decimal/exponent.",
    "FormatFloat": "strconv::FormatFloat(f: f64) -> String": "Default rendering.",
    "FormatBool": "strconv::FormatBool(b: bool) -> String": "\"true\"/\"false\".",
    "ParseBool": "strconv::ParseBool(s: String) -> ParseBoolResult": "Go spellings.",
    "Quote": "strconv::Quote(s: String) -> String": "Double-quoted literal.",
    "Unquote": "strconv::Unquote(s: String) -> UnquoteResult": "Decodes a quoted literal.",
]);

const STD_MATH: &[Item] = stdmod!("math", "Float64 math (`import std::math;`).", [
    "Sqrt": "math::Sqrt(x: f64) -> f64": "Native square root.",
    "Pow": "math::Pow(x: f64, y: f64) -> f64": "Native power.",
    "Exp": "math::Exp(x: f64) -> f64": "Native e^x.",
    "Ln": "math::Ln(x: f64) -> f64": "Native natural log.",
    "Log10": "math::Log10(x: f64) -> f64": "Base-10 log.",
    "Sin": "math::Sin(x: f64) -> f64": "Native sine.",
    "Cos": "math::Cos(x: f64) -> f64": "Native cosine.",
    "Hypot": "math::Hypot(x: f64, y: f64) -> f64": "Hypotenuse.",
    "Floor": "math::Floor(x: f64) -> f64": "Round down.",
    "Ceil": "math::Ceil(x: f64) -> f64": "Round up.",
    "Round": "math::Round(x: f64) -> f64": "Half away from zero.",
    "Max": "math::Max(x: f64, y: f64) -> f64": "Larger.",
    "Min": "math::Min(x: f64, y: f64) -> f64": "Smaller.",
    "Clamp": "math::Clamp(x: f64, lo: f64, hi: f64) -> f64": "Clamps to range.",
    "Pi": "math::Pi -> f64": "Constant π (module const, no call).",
    "E": "math::E -> f64": "Constant e (module const, no call).",
]);

const STD_SORT: &[Item] = stdmod!("sort", "Sorting (`import std::sort;`).", [
    "Ints": "sort::Ints(xs) -> [i64]": "Sorted copy, native.",
    "Floats": "sort::Floats(xs) -> [f64]": "Sorted copy, native.",
    "Strings": "sort::Strings(xs) -> [String]": "Sorted copy, native.",
    "SearchInts": "sort::SearchInts(xs: [i64], x: i64) -> i64": "Lower-bound binary search.",
    "IntsAreSorted": "sort::IntsAreSorted(xs) -> bool": "Sortedness test.",
]);

const STD_SLICES: &[Item] = stdmod!("slices", "List utilities (`import std::slices;`).", [
    "Index": "slices::Index(xs, v) -> i64": "First index or -1.",
    "Contains": "slices::Contains(xs, v) -> bool": "Membership.",
    "Clone": "slices::Clone(xs) -> list": "Shallow copy.",
    "Reverse": "slices::Reverse(xs) -> list": "Reversed copy.",
    "Min": "slices::Min(xs)": "Smallest element.",
    "Max": "slices::Max(xs)": "Largest element.",
    "Insert": "slices::Insert(xs, i: i64, v) -> list": "Inserts at `i`.",
    "Delete": "slices::Delete(xs, i: i64, j: i64) -> list": "Removes `[i:j)`.",
    "Equal": "slices::Equal(a, b) -> bool": "Element equality.",
    "Compact": "slices::Compact(xs) -> list": "Drops consecutive dupes.",
    "Concat": "slices::Concat(xss) -> list": "Concatenates lists.",
    "Map": "slices::Map(xs, |x| …) -> list": "Maps a lambda.",
    "Filter": "slices::Filter(xs, |x| …) -> list": "Keeps truthy.",
    "SortFunc": "slices::SortFunc(xs, |a, b| …) -> list": "Sorts with `less`.",
]);

const STD_ERRORS: &[Item] = stdmod!("errors", "Error values (`import std::errors;`).", [
    "New": "errors::New(msg: String) -> Error": "Creates an error value.",
    "Is": "errors::Is(a: Error, b: Error) -> bool": "Message-identity equality.",
    "Message": "errors::Message(e: Error) -> String": "Carried message.",
]);

const STD_PATH: &[Item] = stdmod!("path", "Slash paths (`import std::path;`).", [
    "Join": "path::Join(elems) -> String": "Joins + cleans.",
    "Split": "path::Split(p: String) -> [dir, file]": "Splits at last slash.",
    "Base": "path::Base(p: String) -> String": "Last element.",
    "Dir": "path::Dir(p: String) -> String": "All but last.",
    "Ext": "path::Ext(p: String) -> String": "Extension.",
    "Clean": "path::Clean(p: String) -> String": "Resolves `.`/`..`.",
    "IsAbs": "path::IsAbs(p: String) -> bool": "Absolute test.",
]);

const STD_FMT: &[Item] = stdmod!("fmt", "Formatted I/O (`import std::fmt;`).", [
    "Print": "fmt::Print(args) -> void": "Concatenates to stdout.",
    "Println": "fmt::Println(args) -> void": "Plus newline.",
    "Printf": "fmt::Printf(format: String, args) -> void": "Verbs %v %d %s %f %t %x %q %%.",
    "Sprintf": "fmt::Sprintf(format: String, args) -> String": "Formats to string.",
    "Sprint": "fmt::Sprint(args) -> String": "Concatenates to string.",
]);

const STD_TIME: &[Item] = stdmod!("time", "Wall clock (`import std::time;`).", [
    "Now": "time::Now() -> Time": "Current instant.",
    "Unix": "time::Unix(sec: i64) -> Time": "From Unix seconds.",
    "Sleep": "time::Sleep(ms: i64) -> void": "Suspends (native).",
    "Add": "time::Add(t: Time, d: i64) -> Time": "Adds ms.",
    "Sub": "time::Sub(a: Time, b: Time) -> i64": "Difference in ms.",
    "Before": "time::Before(a: Time, b: Time) -> bool": "Ordering.",
    "After": "time::After(a: Time, b: Time) -> bool": "Ordering.",
    "Format": "time::Format(t: Time, layout: String) -> String": "\"2006-01-02\", \"15:04:05\", \"RFC3339\".",
    "ParseDate": "time::ParseDate(s: String) -> ParseResult": "Parses Format layouts.",
]);

const STD_BYTES: &[Item] = stdmod!("bytes", "Byte lists (`import std::bytes;`).", [
    "FromString": "bytes::FromString(s: String) -> [i64]": "ASCII bytes.",
    "ToString": "bytes::ToString(b) -> String": "Renders bytes.",
    "Equal": "bytes::Equal(a, b) -> bool": "Byte equality.",
    "Compare": "bytes::Compare(a, b) -> i64": "Three-way compare.",
    "Contains": "bytes::Contains(b, sub) -> bool": "Sub-slice test.",
    "Index": "bytes::Index(b, sub) -> i64": "First index or -1.",
    "Join": "bytes::Join(elems, sep) -> [i64]": "Joins with separator.",
    "Split": "bytes::Split(b, sep) -> [[i64]]": "Splits around `sep`.",
    "ToUpper": "bytes::ToUpper(b) -> [i64]": "ASCII upper.",
    "ToLower": "bytes::ToLower(b) -> [i64]": "ASCII lower.",
]);

const STD_IO: &[Item] = stdmod!("io", "Streams (`import std::io;`).", [
    "NewReader": "io::NewReader(data: String) -> Reader": "In-memory stream.",
    "NewWriter": "io::NewWriter() -> Writer": "Empty sink.",
    "Read": "io::Read(r: Reader, n: i64) -> ReadResult": "`{ reader, chunk, eof }` — reassign `r`.",
    "ReadAll": "io::ReadAll(r: Reader) -> String": "Drains the stream.",
    "Write": "io::Write(w: Writer, s: String) -> WriteResult": "Appends; reassign `w`.",
    "Copy": "io::Copy(w: Writer, r: Reader) -> CopyResult": "Copies all.",
]);

const STD_BUFIO: &[Item] = stdmod!("bufio", "Buffered I/O (`import std::bufio;`).", [
    "NewReader": "bufio::NewReader(r) -> BufReader": "Wraps an `io.Reader`.",
    "ReadString": "bufio::ReadString(r: BufReader, delim: String) -> ReadStringResult": "Reads through `delim`.",
    "NewScanner": "bufio::NewScanner(s: String) -> Scanner": "Line scanner.",
    "NewWordScanner": "bufio::NewWordScanner(s: String) -> Scanner": "Word scanner.",
    "Next": "bufio::Next(s: Scanner) -> ScannerResult": "`{ scanner, text, ok }`.",
]);

const STD_MAPS: &[Item] = stdmod!("maps", "Pair-list maps (`import std::maps;`).", [
    "New": "maps::New() -> map": "Empty map.",
    "Get": "maps::Get(m, k) -> GetResult": "`{ value, ok }`.",
    "Set": "maps::Set(m, k, v) -> map": "Inserts (reassign).",
    "Delete": "maps::Delete(m, k) -> map": "Removes (reassign).",
    "Has": "maps::Has(m, k) -> bool": "Membership.",
    "Keys": "maps::Keys(m) -> list": "Insertion-order keys.",
    "Values": "maps::Values(m) -> list": "Insertion-order values.",
    "Equal": "maps::Equal(a, b) -> bool": "Order-insensitive equality.",
    "Merge": "maps::Merge(a, b) -> map": "`b` wins.",
]);

const STD_CMP: &[Item] = stdmod!("cmp", "Comparisons (`import std::cmp;`).", [
    "CompareInt": "cmp::CompareInt(a: i64, b: i64) -> i64": "-1/0/+1.",
    "CompareFloat": "cmp::CompareFloat(a: f64, b: f64) -> i64": "-1/0/+1.",
    "CompareString": "cmp::CompareString(a: String, b: String) -> i64": "-1/0/+1.",
    "Less": "cmp::Less(a, b) -> bool": "Generic `<`.",
    "Or": "cmp::Or(vals) -> i64": "First non-zero.",
]);

const STD_ITER: &[Item] = stdmod!("iter", "Iterator adapters (`import std::iter;`).", [
    "Map": "iter::Map(xs, |x| …)": "Maps a lambda.",
    "Filter": "iter::Filter(xs, |x| …)": "Keeps truthy.",
    "Reduce": "iter::Reduce(xs, init, |acc, x| …)": "Left fold.",
    "Take": "iter::Take(xs, n: i64) -> list": "First `n`.",
    "Drop": "iter::Drop(xs, n: i64) -> list": "After first `n`.",
    "Chain": "iter::Chain(a, b) -> list": "Concatenates.",
    "Next": "iter::Next(it: Iter) -> NextResult": "Pulls one element.",
]);

const STD_UTF8: &[Item] = stdmod!("utf8", "Runes (`import std::unicode::utf8;` → key `utf8`).", [
    "RuneCount": "utf8::RuneCount(s: String) -> i64": "Rune count.",
    "DecodeRune": "utf8::DecodeRune(s: String) -> RuneDecoded": "`{ rune, size, ok }`.",
    "EncodeRune": "utf8::EncodeRune(r: i64) -> String": "Encodes a code point.",
    "RuneLen": "utf8::RuneLen(r: i64) -> i64": "UTF-8 width.",
    "ValidString": "utf8::ValidString(s: String) -> bool": "Always true for runtime strings.",
]);

const STD_LIST: &[Item] = stdmod!("list", "Linked-list vocabulary (`import std::container::list;` → key `list`).", [
    "New": "list::New() -> List": "Empty list.",
    "PushBack": "list::PushBack(l: List, v) -> List": "Appends (reassign).",
    "PushFront": "list::PushFront(l: List, v) -> List": "Prepends (reassign).",
    "Front": "list::Front(l: List)": "First element.",
    "Back": "list::Back(l: List)": "Last element.",
    "PopBack": "list::PopBack(l: List) -> ListPopResult": "`{ list, value, ok }`.",
    "Len": "list::Len(l: List) -> i64": "Element count.",
    "ToList": "list::ToList(l: List) -> list": "Plain list.",
]);

const STD_HEAP: &[Item] = stdmod!("heap", "Binary heap (`import std::container::heap;` → key `heap`).", [
    "New": "heap::New(|a, b| …) -> Heap": "Empty heap with `less` stored inside.",
    "Push": "heap::Push(h: Heap, v) -> Heap": "Pushes (reassign).",
    "Pop": "heap::Pop(h: Heap) -> HeapPopResult": "`{ heap, value, ok }`.",
    "Peek": "heap::Peek(h: Heap)": "Top without removing.",
]);

const STD_RING: &[Item] = stdmod!("ring", "Circular ring (`import std::container::ring;` → key `ring`).", [
    "New": "ring::New(values) -> Ring": "Ring over elements.",
    "Value": "ring::Value(r: Ring)": "Current element.",
    "Next": "ring::Next(r: Ring) -> Ring": "Cursor forward (reassign).",
    "Prev": "ring::Prev(r: Ring) -> Ring": "Cursor back (reassign).",
    "Set": "ring::Set(r: Ring, v) -> Ring": "Replaces current.",
    "Do": "ring::Do(r: Ring, |v| …) -> void": "Calls `f` per element.",
]);

const STD_OS: &[Item] = stdmod!("os", "OS surface (`import std::os;`).", [
    "Args": "os::Args() -> [String]": "CLI arguments.",
    "Getenv": "os::Getenv(k: String) -> String": "`\"\"` when unset.",
    "Setenv": "os::Setenv(k: String, v: String) -> void": "Sets a variable.",
    "LookupEnv": "os::LookupEnv(k: String) -> [value, ok]": "Value + presence.",
    "ReadFile": "os::ReadFile(p: String) -> String": "Whole file.",
    "WriteFile": "os::WriteFile(p: String, data: String) -> bool": "Whole file write.",
    "Getwd": "os::Getwd() -> String": "Working directory.",
    "TempDir": "os::TempDir() -> String": "Temp directory.",
    "MkdirAll": "os::MkdirAll(p: String) -> bool": "Makes parents.",
    "ReadDir": "os::ReadDir(d: String) -> [String]": "Entry names.",
    "Stat": "os::Stat(p: String) -> FileInfo": "`{ name, size, is_dir }`.",
    "Exit": "os::Exit(code: i64) -> void": "Terminates.",
]);

const STD_EXEC: &[Item] = stdmod!("exec", "Subprocesses (`import std::os::exec;` → key `exec`).", [
    "Command": "exec::Command(prog: String, argv) -> Cmd": "Prepares a command.",
    "Output": "exec::Output(c: Cmd) -> OutputResult": "`{ out, err, code, ok }` (native capture).",
    "Run": "exec::Run(c: Cmd) -> bool": "Success only.",
    "CombinedOutput": "exec::CombinedOutput(c: Cmd) -> OutputResult": "stdout+stderr.",
    "LookPath": "exec::LookPath(prog: String) -> String": "Resolves on PATH.",
]);

const STD_SYNC: &[Item] = stdmod!("sync", "Synchronization (`import std::sync;`).", [
    "NewMutex": "sync::NewMutex() -> Mutex": "Unlocked mutex.",
    "Lock": "sync::Lock(m: Mutex) -> Mutex": "Locks (reassign).",
    "Unlock": "sync::Unlock(m: Mutex) -> Mutex": "Unlocks (reassign).",
    "WithLock": "sync::WithLock(m: Mutex, | | …) -> Mutex": "Runs `f` held.",
    "NewWaitGroup": "sync::NewWaitGroup() -> WaitGroup": "Empty gate.",
    "Add": "sync::Add(w: WaitGroup, d: i64) -> WaitGroup": "Adds delta.",
    "Done": "sync::Done(w: WaitGroup) -> WaitGroup": "Marks one done.",
    "NewOnce": "sync::NewOnce() -> Once": "Untriggered once.",
    "Do": "sync::Do(o: Once, | | …) -> Once": "Runs `f` first time only.",
]);

const STD_ATOMIC: &[Item] = stdmod!("atomic", "Atomic int64 (`import std::sync::atomic;` → key `atomic`).", [
    "NewInt64": "atomic::NewInt64(v: i64) -> Int64": "Allocates a cell (native).",
    "Load": "atomic::Load(a: Int64) -> i64": "Atomic load.",
    "Add": "atomic::Add(a: Int64, d: i64) -> i64": "Returns new value.",
    "Swap": "atomic::Swap(a: Int64, v: i64) -> i64": "Returns old value.",
    "CompareAndSwap": "atomic::CompareAndSwap(a: Int64, exp: i64, next: i64) -> bool": "CAS.",
]);

const STD_CONTEXT: &[Item] = stdmod!("context", "Request contexts (`import std::context;`).", [
    "Background": "context::Background() -> Context": "Empty context.",
    "WithCancel": "context::WithCancel(p: Context) -> CancelResult": "`{ ctx, cancel }`; call `cancel()`, reassign.",
    "Cancel": "context::Cancel(c: Context) -> Context": "Marks cancelled.",
    "WithTimeout": "context::WithTimeout(p: Context, ms: i64) -> CancelResult": "Deadline child.",
    "WithValue": "context::WithValue(p: Context, k, v) -> Context": "Carries a value.",
    "Value": "context::Value(c: Context, k) -> ValueResult": "`{ value, ok }`.",
    "Done": "context::Done(c: Context) -> bool": "Cancelled or expired.",
    "Err": "context::Err(c: Context) -> String": "\"\"/\"Canceled\"/\"DeadlineExceeded\".",
]);

const STD_LOG: &[Item] = stdmod!("log", "Default logger (`import std::log;`).", [
    "Print": "log::Print(args) -> void": "One line.",
    "Println": "log::Println(args) -> void": "One line.",
    "Printf": "log::Printf(format: String, args) -> void": "Formatted line.",
    "SetPrefix": "log::SetPrefix(p: String) -> void": "Sets the prefix.",
    "Fatal": "log::Fatal(args) -> void": "Logs + exits 1.",
    "Panic": "log::Panic(args) -> void": "Aborts with message.",
]);

const STD_FILEPATH: &[Item] = stdmod!("filepath", "OS paths (`import std::path::filepath;` → key `filepath`).", [
    "Join": "filepath::Join(elems) -> String": "Joins with OS separator.",
    "Base": "filepath::Base(p: String) -> String": "Last element.",
    "Dir": "filepath::Dir(p: String) -> String": "All but last.",
    "Ext": "filepath::Ext(p: String) -> String": "Extension.",
    "Clean": "filepath::Clean(p: String) -> String": "Cleans.",
    "IsAbs": "filepath::IsAbs(p: String) -> bool": "Absolute (drive-aware).",
    "Walk": "filepath::Walk(dir: String) -> [String]": "Recursive listing.",
    "Match": "filepath::Match(pattern: String, name: String) -> bool": "Glob `*`/`?`.",
    "Separator": "filepath::Separator() -> String": "OS separator.",
]);

const STD_JSON: &[Item] = stdmod!("json", "JSON codec (`import std::encoding::json;` → key `json`).", [
    "Marshal": "json::Marshal(v) -> String": "Serializes (native).",
    "Unmarshal": "json::Unmarshal(s: String)": "Parses (native).",
    "Valid": "json::Valid(s: String) -> bool": "Well-formed test (native).",
]);

const STD_BASE64: &[Item] = stdmod!("base64", "Base64 (`import std::encoding::base64;` → key `base64`).", [
    "Encode": "base64::Encode(s: String) -> String": "Std alphabet + padding.",
    "EncodeURL": "base64::EncodeURL(s: String) -> String": "URL-safe alphabet.",
    "Decode": "base64::Decode(s: String) -> B64Decoded": "`{ text, ok }`.",
    "DecodeURL": "base64::DecodeURL(s: String) -> B64Decoded": "URL-safe decode.",
]);

const STD_BASE32: &[Item] = stdmod!("base32", "Base32 (`import std::encoding::base32;` → key `base32`).", [
    "Encode": "base32::Encode(s: String) -> String": "Std alphabet + padding.",
    "Decode": "base32::Decode(s: String) -> DecodeResult32": "`{ text, ok }`.",
]);

const STD_HEX: &[Item] = stdmod!("hex", "Hex codec (`import std::encoding::hex;` → key `hex`).", [
    "Encode": "hex::Encode(s: String) -> String": "Lowercase hex.",
    "Decode": "hex::Decode(s: String) -> HexDecoded": "`{ text, ok }`.",
]);

const STD_CSV: &[Item] = stdmod!("csv", "CSV (`import std::encoding::csv;` → key `csv`).", [
    "ReadAll": "csv::ReadAll(s: String) -> [[String]]": "RFC-4180 rows.",
    "WriteAll": "csv::WriteAll(rows) -> String": "Serializes rows.",
]);

const STD_HTML: &[Item] = stdmod!("html", "HTML escaping (`import std::html;`).", [
    "EscapeString": "html::EscapeString(s: String) -> String": "Escapes `&<>'\"`.",
    "UnescapeString": "html::UnescapeString(s: String) -> String": "Unescapes entities.",
]);

const STD_REGEXP: &[Item] = stdmod!("regexp", "Regex subset (`import std::regexp;`).", [
    "Match": "regexp::Match(pattern: String, s: String) -> bool": "Partial match.",
    "Find": "regexp::Find(pattern: String, s: String) -> FindResult": "`{ text, start, end, ok }`.",
    "FindAll": "regexp::FindAll(pattern: String, s: String) -> [String]": "All matches.",
    "ReplaceAll": "regexp::ReplaceAll(s: String, pattern: String, repl: String) -> String": "Literal replacement.",
    "Split": "regexp::Split(s: String, pattern: String) -> [String]": "Splits around matches.",
    "QuoteMeta": "regexp::QuoteMeta(s: String) -> String": "Escapes meta chars.",
]);

const STD_FNV: &[Item] = stdmod!("fnv", "FNV hashes (`import std::hash::fnv;` → key `fnv`).", [
    "Sum32a": "fnv::Sum32a(s: String) -> i64": "One-shot 32-bit (native).",
    "Sum64a": "fnv::Sum64a(s: String) -> i64": "One-shot 64-bit (native).",
    "New32a": "fnv::New32a() -> Hash32": "Streaming state.",
    "Write32": "fnv::Write32(h: Hash32, s: String) -> Hash32": "Feeds (reassign).",
    "Sum32": "fnv::Sum32(h: Hash32) -> i64": "Finalizes.",
]);

const STD_CRC32: &[Item] = stdmod!("crc32", "CRC-32 (`import std::hash::crc32;` → key `crc32`).", [
    "Checksum": "crc32::Checksum(s: String) -> i64": "IEEE one-shot (native).",
    "New": "crc32::New() -> CrcHash": "Streaming state.",
    "Write": "crc32::Write(h: CrcHash, s: String) -> CrcHash": "Feeds (reassign).",
    "Sum32": "crc32::Sum32(h: CrcHash) -> i64": "Finalizes.",
]);

const STD_ADLER32: &[Item] = stdmod!("adler32", "Adler-32 (`import std::hash::adler32;` → key `adler32`).", [
    "Checksum": "adler32::Checksum(s: String) -> i64": "One-shot (native).",
    "New": "adler32::New() -> AdlerHash": "Streaming state.",
    "Write": "adler32::Write(h: AdlerHash, s: String) -> AdlerHash": "Feeds (reassign).",
    "Sum32": "adler32::Sum32(h: AdlerHash) -> i64": "Finalizes.",
]);

const STD_RAND: &[Item] = stdmod!("rand", "Random numbers (`import std::rand;`).", [
    "Seed": "rand::Seed(s: i64) -> void": "Reseeds (native).",
    "Intn": "rand::Intn(n: i64) -> i64": "Uniform [0, n).",
    "Float64": "rand::Float64() -> f64": "Uniform [0, 1).",
    "Shuffle": "rand::Shuffle(xs) -> list": "Fisher-Yates copy.",
    "Choice": "rand::Choice(xs)": "Uniform element.",
    "Range": "rand::Range(lo: i64, hi: i64) -> i64": "Uniform [lo, hi).",
]);

const STD_SUBTLE: &[Item] = stdmod!("subtle", "Constant-time helpers (`import std::crypto::subtle;` → key `subtle`).", [
    "ConstantTimeCompare": "subtle::ConstantTimeCompare(a: String, b: String) -> i64": "1 when equal.",
    "ConstantTimeSelect": "subtle::ConstantTimeSelect(x: i64, v0: i64, v1: i64) -> i64": "Selects without branching.",
]);

const STD_SHA256: &[Item] = stdmod!("sha256", "SHA-256 (`import std::crypto::sha256;` → key `sha256`).", [
    "Sum": "sha256::Sum(s: String) -> String": "Hex digest (FIPS vectors).",
    "SumBytes": "sha256::SumBytes(s: String) -> [i64]": "32 digest bytes.",
]);

const STD_HMAC: &[Item] = stdmod!("hmac", "HMAC (`import std::crypto::hmac;` → key `hmac`).", [
    "Sha256": "hmac::Sha256(key: String, msg: String) -> String": "Hex (RFC 4231 vector).",
    "Sha256Bytes": "hmac::Sha256Bytes(key: String, msg: String) -> [i64]": "32 raw bytes.",
]);

const STD_SLOG: &[Item] = stdmod!("slog", "Structured logging (`import std::slog;`).", [
    "Info": "slog::Info(l: Logger, msg: String, attrs) -> void": "Info record.",
    "Debug": "slog::Debug(l: Logger, msg: String, attrs) -> void": "Debug record.",
    "Warn": "slog::Warn(l: Logger, msg: String, attrs) -> void": "Warn record.",
    "Error": "slog::Error(l: Logger, msg: String, attrs) -> void": "Error record.",
    "New": "slog::New(level: i64) -> Logger": "Logger at level.",
    "With": "slog::With(l: Logger, attrs) -> Logger": "Binds attributes.",
    "Inf": "slog::Inf(msg: String) -> void": "Default-logger Info.",
]);

const STD_FLAG: &[Item] = stdmod!("flag", "CLI flags (`import std::flag;`).", [
    "String": "flag::String(name: String, def: String, usage: String) -> void": "Declares a string flag.",
    "Int": "flag::Int(name: String, def: i64, usage: String) -> void": "Declares an int flag.",
    "Bool": "flag::Bool(name: String, def: bool, usage: String) -> void": "Declares a bool flag.",
    "Parse": "flag::Parse() -> void": "Parses `Process::args()`.",
    "GetString": "flag::GetString(name: String) -> String": "String value.",
    "GetInt": "flag::GetInt(name: String) -> i64": "Int value.",
    "GetBool": "flag::GetBool(name: String) -> bool": "Bool value.",
    "Usage": "flag::Usage() -> void": "Prints usage.",
]);

const STD_MIME: &[Item] = stdmod!("mime", "Media types (`import std::mime;`).", [
    "ParseMediaType": "mime::ParseMediaType(v: String) -> MediaResult": "`{ typ, params, ok }`.",
    "FormatMediaType": "mime::FormatMediaType(t: String, params) -> String": "Formats back.",
    "TypeByExtension": "mime::TypeByExtension(ext: String) -> String": "Guesses from extension.",
]);

const STD_URL: &[Item] = stdmod!("url", "URLs (`import std::net::url;` → key `url`).", [
    "Parse": "url::Parse(raw: String) -> ParseResult": "`{ url, ok }` (scheme/host/port/path/query/fragment).",
    "StringOf": "url::StringOf(u: URL) -> String": "Renders back.",
    "ParseQuery": "url::ParseQuery(q: String) -> [[k, v]]": "Parses query pairs.",
    "EncodeQuery": "url::EncodeQuery(pairs) -> String": "Encodes pairs.",
    "QueryEscape": "url::QueryEscape(s: String) -> String": "Percent-encodes (UTF-8).",
    "QueryUnescape": "url::QueryUnescape(s: String) -> String": "Decodes +/%XX (UTF-8).",
]);

const STD_BITS: &[Item] = stdmod!("bits", "Bit counting (`import std::math::bits;` → key `bits`).", [    "OnesCount": "bits::OnesCount(x: i64) -> i64": "One bits (64-bit view).",
    "TrailingZeros": "bits::TrailingZeros(x: i64) -> i64": "Trailing zeros (64 if 0).",
    "LeadingZeros": "bits::LeadingZeros(x: i64) -> i64": "Leading zeros.",
    "Len": "bits::Len(x: i64) -> i64": "Bit length.",
    "RotateLeft": "bits::RotateLeft(x: i64, k: i64) -> i64": "Rotates (k mod 64).",
    "Reverse": "bits::Reverse(x: i64) -> i64": "Reverses bit order.",
    "ReverseBytes": "bits::ReverseBytes(x: i64) -> i64": "Reverses byte order.",
]);

const STD_UNSAFE: &[Item] = stdmod!("unsafe", "Safe subset (`import std::unsafe;`).", [
    "Sizeof": "unsafe::Sizeof(v) -> i64": "Word-model size.",
    "Alignof": "unsafe::Alignof(v) -> i64": "Always 8.",
    "IsNil": "unsafe::IsNil(v) -> bool": "Null test.",
]);

const STD_RUNTIME: &[Item] = stdmod!("runtime", "Runtime facts (`import std::runtime;`).", [
    "Version": "runtime::Version() -> String": "Toolchain version.",
    "GOOS": "runtime::GOOS() -> String": "OS (native).",
    "GOARCH": "runtime::GOARCH() -> String": "Arch (native).",
    "NumCPU": "runtime::NumCPU() -> i64": "Logical CPUs (native).",
    "NumGoroutine": "runtime::NumGoroutine() -> i64": "Always 1 (single task).",
    "GOMAXPROCS": "runtime::GOMAXPROCS(n: i64) -> i64": "Records max threads.",
]);

const STD_TESTING: &[Item] = stdmod!("testing", "Test helpers (`import std::testing;`).", [
    "New": "testing::New(name: String) -> T": "Test context.",
    "Assert": "testing::Assert(t: T, cond: bool, msg: String) -> T": "Boolean assertion.",
    "AssertEq": "testing::AssertEq(t: T, a, b, msg: String) -> T": "Equality assertion.",
    "Failed": "testing::Failed(t: T) -> bool": "Failure state.",
    "Benchmark": "testing::Benchmark(n: i64, | | …) -> BenchResult": "Times `n` runs.",
]);

const STD_MAIL: &[Item] = stdmod!("mail", "Addresses (`import std::net::mail;` → key `mail`).", [
    "ParseAddress": "mail::ParseAddress(s: String) -> ParseResult": "`{ addr, ok }`.",
    "ParseAddressList": "mail::ParseAddressList(s: String) -> [Address]": "Comma list.",
    "StringOf": "mail::StringOf(a: Address) -> String": "Renders back.",
]);

const STD_TEXTPROTO: &[Item] = stdmod!("textproto", "Header reading (`import std::net::textproto;` → key `textproto`).", [
    "ReadMIMEHeader": "textproto::ReadMIMEHeader(s: String) -> HeaderResult": "`{ header, body }`.",
    "CanonicalKey": "textproto::CanonicalKey(s: String) -> String": "MIME key form.",
    "Get": "textproto::Get(header, key: String) -> String": "First value.",
]);

const STD_MULTIPART: &[Item] = stdmod!("multipart", "Multipart (`import std::mime::multipart;` → key `multipart`).", [
    "Parse": "multipart::Parse(s: String, boundary: String) -> ParseResult": "`{ parts, ok }`.",
    "HeaderGet": "multipart::HeaderGet(p: Part, key: String) -> String": "Part header.",
    "FileName": "multipart::FileName(p: Part) -> String": "Disposition filename.",
]);

const STD_PEM: &[Item] = stdmod!("pem", "PEM blocks (`import std::encoding::pem;` → key `pem`).", [
    "Decode": "pem::Decode(s: String) -> DecodeResult": "`{ block, rest, ok }`.",
    "Encode": "pem::Encode(typ: String, data: String) -> String": "Wraps base64.",
]);

const STD_ASCII85: &[Item] = stdmod!("ascii85", "Ascii85 (`import std::encoding::ascii85;` → key `ascii85`).", [
    "Encode": "ascii85::Encode(s: String) -> String": "With z/y shortcuts.",
    "Decode": "ascii85::Decode(s: String) -> DecodeResult85": "`{ text, ok }`.",
]);

const STD_COLOR: &[Item] = stdmod!("color", "Colors (`import std::image::color;` → key `color`).", [
    "ParseHex": "color::ParseHex(s: String) -> RGBA": "Parses #rgb/#rrggbb.",
    "ToHex": "color::ToHex(c: RGBA) -> String": "Renders #rrggbb.",
    "ToGray": "color::ToGray(c: RGBA) -> Gray": "Rec. 601 luma.",
    "NRGBAtoRGBA": "color::NRGBAtoRGBA(c: NRGBA) -> RGBA": "Premultiplies.",
]);

const STD_TAR: &[Item] = stdmod!("tar", "Ustar archives (`import std::archive::tar;` → key `tar`).", [
    "NewReader": "tar::NewReader(data: String) -> Reader": "Archive reader.",
    "Next": "tar::Next(r: Reader) -> NextResult": "`{ reader, header, data, ok }`.",
    "AppendFile": "tar::AppendFile(ar: String, name: String, data: String, mode: i64) -> String": "Adds a file.",
    "AppendDir": "tar::AppendDir(ar: String, name: String, mode: i64) -> String": "Adds a dir.",
]);

const STD_CRC64: &[Item] = stdmod!("crc64", "CRC-64 (`import std::hash::crc64;` → key `crc64`).", [
    "Checksum": "crc64::Checksum(s: String) -> i64": "ECMA one-shot (native).",
    "New": "crc64::New() -> Hash64State": "Streaming state.",
    "Write": "crc64::Write(h: Hash64State, s: String) -> Hash64State": "Feeds (reassign).",
    "Sum64": "crc64::Sum64(h: Hash64State) -> i64": "Finalizes.",
]);

const STD_MAPHASH: &[Item] = stdmod!("maphash", "Seeded hashes (`import std::hash::maphash;` → key `maphash`).", [
    "MakeSeed": "maphash::MakeSeed(n: i64) -> Seed": "Builds a seed.",
    "String": "maphash::String(seed: Seed, s: String) -> i64": "Seeded string hash.",
    "Int": "maphash::Int(seed: Seed, v: i64) -> i64": "Splitmix64 int hash.",
]);

const STD_SIGNAL: &[Item] = stdmod!("signal", "Signals (`import std::os::signal;` → key `signal`).", [
    "Notify": "signal::Notify(sig: i64) -> void": "Registers interest.",
    "Stop": "signal::Stop(sig: i64) -> void": "Withdraws interest.",
    "Wanted": "signal::Wanted() -> [i64]": "Registered signals.",
]);

const STD_USER: &[Item] = stdmod!("user", "Users (`import std::os::user;` → key `user`).", [
    "Current": "user::Current() -> User": "Current user (env).",
    "Lookup": "user::Lookup(name: String) -> [User, bool]": "Matches current only.",
]);

const STD_EXPVAR: &[Item] = stdmod!("expvar", "Exported vars (`import std::expvar;`).", [
    "NewInt": "expvar::NewInt(name: String) -> void": "Publishes int.",
    "Add": "expvar::Add(name: String, d: i64) -> void": "Adds delta.",
    "Set": "expvar::Set(name: String, v) -> void": "Sets value.",
    "Get": "expvar::Get(name: String)": "Reads value.",
    "Render": "expvar::Render() -> String": "JSON for /debug/vars.",
]);

const STD_UTF16: &[Item] = stdmod!("utf16", "Surrogates (`import std::unicode::utf16;` → key `utf16`).", [
    "Encode": "utf16::Encode(s: String) -> [i64]": "String to units.",
    "Decode": "utf16::Decode(units) -> String": "Units to string.",
    "EncodeRune": "utf16::EncodeRune(r: i64) -> [i64]": "Rune to units.",
    "DecodeRune": "utf16::DecodeRune(hi: i64, lo: i64) -> i64": "Pair to rune.",
]);

const SYS_MEMBERS: &[Item] = &[
    Item { label: "goos", kind: Kind::Function, detail: "Sys::goos() -> String", doc: "Host OS (`windows`/`linux`/`macos`, backs `std::runtime`).", insert: "goos()" },
    Item { label: "arch", kind: Kind::Function, detail: "Sys::arch() -> String", doc: "Host arch (`x86_64`/`aarch64`).", insert: "arch()" },
    Item { label: "ncpu", kind: Kind::Function, detail: "Sys::ncpu() -> i64", doc: "Logical CPU count.", insert: "ncpu()" },
    Item { label: "lex_version", kind: Kind::Function, detail: "Sys::lex_version() -> String", doc: "Toolchain version baked at compile time (tracks release auto-bumps).", insert: "lex_version()" },
];

pub const MODULES: &[Module] = &[
    Module { name: "Console", doc: "Stdout printing (native, no import).", members: CONSOLE_MEMBERS },
    Module { name: "Json", doc: "Runtime JSON codec (native, no import).", members: JSON_MEMBERS },
    Module { name: "Env", doc: "Environment variables (native, no import).", members: ENV_MEMBERS },
    Module { name: "Http", doc: "Real HTTP server + blocking fetch (native, no import).", members: HTTP_MEMBERS },
    Module { name: "Time", doc: "Wall clock (native, no import; see also `std::time`).", members: TIME_MEMBERS },
    Module { name: "File", doc: "Filesystem (native, no import; see also `std::os`).", members: FILE_MEMBERS },
    Module { name: "Process", doc: "Args/exit/spawn (native, no import; see also `std::os::exec`).", members: PROCESS_MEMBERS },
    Module { name: "Text", doc: "Char-level string primitives (native, no import).", members: TEXT_MEMBERS },
    Module { name: "Math", doc: "Exact float64 math (native, no import; see also `std::math`).", members: MATH_MEMBERS },
    Module { name: "List", doc: "Native list algorithms (no import).", members: LIST_MEMBERS },
    Module { name: "Hash", doc: "Native one-shot hashes (no import).", members: HASH_MEMBERS },
    Module { name: "Rand", doc: "Native PRNG (no import; see also `std::rand`).", members: RAND_MEMBERS },
    Module { name: "Atomic", doc: "Shared atomic-i64 cells (native, no import; see also `std::sync::atomic`).", members: ATOMIC_MEMBERS },
    Module { name: "Sys", doc: "Host facts (native, no import; see also `std::runtime`).", members: SYS_MEMBERS },
    Module { name: "strings", doc: "`import std::strings;` — Go-parity strings.", members: STD_STRINGS },
    Module { name: "strconv", doc: "`import std::strconv;` — conversions.", members: STD_STRCONV },
    Module { name: "math", doc: "`import std::math;` — float64 math + consts.", members: STD_MATH },
    Module { name: "sort", doc: "`import std::sort;` — native sorts + search.", members: STD_SORT },
    Module { name: "slices", doc: "`import std::slices;` — list utilities + lambdas.", members: STD_SLICES },
    Module { name: "errors", doc: "`import std::errors;` — error values.", members: STD_ERRORS },
    Module { name: "path", doc: "`import std::path;` — slash paths.", members: STD_PATH },
    Module { name: "fmt", doc: "`import std::fmt;` — formatted I/O.", members: STD_FMT },
    Module { name: "time", doc: "`import std::time;` — wall clock + Format/Parse.", members: STD_TIME },
    Module { name: "bytes", doc: "`import std::bytes;` — byte lists.", members: STD_BYTES },
    Module { name: "io", doc: "`import std::io;` — value-semantics streams.", members: STD_IO },
    Module { name: "bufio", doc: "`import std::bufio;` — buffered reading + Scanner.", members: STD_BUFIO },
    Module { name: "maps", doc: "`import std::maps;` — pair-list maps.", members: STD_MAPS },
    Module { name: "cmp", doc: "`import std::cmp;` — three-way comparisons.", members: STD_CMP },
    Module { name: "iter", doc: "`import std::iter;` — adapters over lambdas.", members: STD_ITER },
    Module { name: "utf8", doc: "`import std::unicode::utf8;` — runes.", members: STD_UTF8 },
    Module { name: "list", doc: "`import std::container::list;` — list vocabulary.", members: STD_LIST },
    Module { name: "heap", doc: "`import std::container::heap;` — heap with lambda `less`.", members: STD_HEAP },
    Module { name: "ring", doc: "`import std::container::ring;` — circular ring.", members: STD_RING },
    Module { name: "os", doc: "`import std::os;` — args/env/files.", members: STD_OS },
    Module { name: "exec", doc: "`import std::os::exec;` — subprocesses.", members: STD_EXEC },
    Module { name: "sync", doc: "`import std::sync;` — Mutex/WaitGroup/Once.", members: STD_SYNC },
    Module { name: "atomic", doc: "`import std::sync::atomic;` — atomic int64 cells.", members: STD_ATOMIC },
    Module { name: "context", doc: "`import std::context;` — cancel/timeout/values.", members: STD_CONTEXT },
    Module { name: "log", doc: "`import std::log;` — default logger.", members: STD_LOG },
    Module { name: "filepath", doc: "`import std::path::filepath;` — OS paths + Walk/glob.", members: STD_FILEPATH },
    Module { name: "json", doc: "`import std::encoding::json;` — Marshal/Unmarshal/Valid.", members: STD_JSON },
    Module { name: "base64", doc: "`import std::encoding::base64;` — Std/URL codecs.", members: STD_BASE64 },
    Module { name: "base32", doc: "`import std::encoding::base32;` — base32 codecs.", members: STD_BASE32 },
    Module { name: "hex", doc: "`import std::encoding::hex;` — hex codec.", members: STD_HEX },
    Module { name: "csv", doc: "`import std::encoding::csv;` — RFC-4180.", members: STD_CSV },
    Module { name: "html", doc: "`import std::html;` — escaping.", members: STD_HTML },
    Module { name: "regexp", doc: "`import std::regexp;` — documented-subset regex.", members: STD_REGEXP },
    Module { name: "fnv", doc: "`import std::hash::fnv;` — FNV-1a.", members: STD_FNV },
    Module { name: "crc32", doc: "`import std::hash::crc32;` — IEEE CRC-32.", members: STD_CRC32 },
    Module { name: "adler32", doc: "`import std::hash::adler32;` — Adler-32.", members: STD_ADLER32 },
    Module { name: "rand", doc: "`import std::rand;` — PRNG + shuffle.", members: STD_RAND },
    Module { name: "subtle", doc: "`import std::crypto::subtle;` — constant-time helpers.", members: STD_SUBTLE },
    Module { name: "sha256", doc: "`import std::crypto::sha256;` — FIPS vectors.", members: STD_SHA256 },
    Module { name: "hmac", doc: "`import std::crypto::hmac;` — HMAC-SHA-256.", members: STD_HMAC },
    Module { name: "slog", doc: "`import std::slog;` — structured logging.", members: STD_SLOG },
    Module { name: "flag", doc: "`import std::flag;` — CLI flags.", members: STD_FLAG },
    Module { name: "mime", doc: "`import std::mime;` — media types.", members: STD_MIME },
    Module { name: "url", doc: "`import std::net::url;` — URL parse + query codec.", members: STD_URL },
    Module { name: "bits", doc: "`import std::math::bits;` — bit counting.", members: STD_BITS },
    Module { name: "unsafe", doc: "`import std::unsafe;` — safe subset (Sizeof/Alignof).", members: STD_UNSAFE },
    Module { name: "runtime", doc: "`import std::runtime;` — GOOS/NumCPU/Version.", members: STD_RUNTIME },
    Module { name: "testing", doc: "`import std::testing;` — T/assertions/bench.", members: STD_TESTING },
    Module { name: "mail", doc: "`import std::net::mail;` — addresses.", members: STD_MAIL },
    Module { name: "textproto", doc: "`import std::net::textproto;` — headers.", members: STD_TEXTPROTO },
    Module { name: "multipart", doc: "`import std::mime::multipart;` — parts.", members: STD_MULTIPART },
    Module { name: "pem", doc: "`import std::encoding::pem;` — PEM blocks.", members: STD_PEM },
    Module { name: "ascii85", doc: "`import std::encoding::ascii85;` — ascii85.", members: STD_ASCII85 },
    Module { name: "color", doc: "`import std::image::color;` — colors.", members: STD_COLOR },
    Module { name: "tar", doc: "`import std::archive::tar;` — ustar.", members: STD_TAR },
    Module { name: "crc64", doc: "`import std::hash::crc64;` — ECMA CRC-64.", members: STD_CRC64 },
    Module { name: "maphash", doc: "`import std::hash::maphash;` — seeded hashes.", members: STD_MAPHASH },
    Module { name: "signal", doc: "`import std::os::signal;` — constants/Notify.", members: STD_SIGNAL },
    Module { name: "user", doc: "`import std::os::user;` — current user.", members: STD_USER },
    Module { name: "expvar", doc: "`import std::expvar;` — exported vars.", members: STD_EXPVAR },
    Module { name: "utf16", doc: "`import std::unicode::utf16;` — surrogates.", members: STD_UTF16 },
];

// ---------------------------------------------------------------------------
// Go-to-definition map: module key → SDK-relative source path. Native
// builtins (PascalCase) resolve to generated `native/*.lex` signature
// stubs (same API, documented as native-backed); file modules resolve to
// their real `lib/std/**/*.lex` sources. The LSP searches these through
// the interpreter's own roots, so Ctrl+Click lands where code runs from.
// ---------------------------------------------------------------------------

/// SDK-relative path segments for every file-module key.
pub const STD_PATHS: &[(&str, &str)] = &[
    ("strings", "std/strings"), ("strconv", "std/strconv"), ("math", "std/math"),
    ("sort", "std/sort"), ("slices", "std/slices"), ("errors", "std/errors"),
    ("path", "std/path"), ("fmt", "std/fmt"), ("time", "std/time"),
    ("bytes", "std/bytes"), ("io", "std/io"), ("bufio", "std/bufio"),
    ("maps", "std/maps"), ("cmp", "std/cmp"), ("iter", "std/iter"),
    ("utf8", "std/unicode/utf8"), ("utf16", "std/unicode/utf16"),
    ("list", "std/container/list"), ("heap", "std/container/heap"),
    ("ring", "std/container/ring"), ("os", "std/os"), ("exec", "std/os/exec"),
    ("signal", "std/os/signal"), ("user", "std/os/user"),
    ("sync", "std/sync"), ("atomic", "std/sync/atomic"),
    ("context", "std/context"), ("log", "std/log"),
    ("filepath", "std/path/filepath"), ("json", "std/encoding/json"),
    ("base64", "std/encoding/base64"), ("base32", "std/encoding/base32"),
    ("hex", "std/encoding/hex"), ("csv", "std/encoding/csv"),
    ("pem", "std/encoding/pem"), ("ascii85", "std/encoding/ascii85"),
    ("html", "std/html"), ("regexp", "std/regexp"),
    ("fnv", "std/hash/fnv"), ("crc32", "std/hash/crc32"),
    ("crc64", "std/hash/crc64"), ("adler32", "std/hash/adler32"),
    ("maphash", "std/hash/maphash"), ("rand", "std/rand"),
    ("subtle", "std/crypto/subtle"), ("sha256", "std/crypto/sha256"),
    ("hmac", "std/crypto/hmac"), ("slog", "std/slog"), ("flag", "std/flag"),
    ("mime", "std/mime"), ("multipart", "std/mime/multipart"),
    ("url", "std/net/url"), ("mail", "std/net/mail"),
    ("textproto", "std/net/textproto"), ("bits", "std/math/bits"),
    ("unsafe", "std/unsafe"), ("runtime", "std/runtime"),
    ("testing", "std/testing"), ("color", "std/image/color"),
    ("tar", "std/archive/tar"),
];

/// Native built-in modules (implemented in Rust, documented by stubs).
pub const NATIVE_MODULES: &[&str] = &[
    "Console", "Json", "Env", "Http", "Time", "File", "Process", "Text",
    "Math", "List", "Hash", "Rand", "Atomic", "Sys",
];

/// Resolve a module key to SDK-relative path segments (without extension).
/// Native modules map to their `native/<lower>` signature stubs.
pub fn module_sdk_path(key: &str) -> Option<Vec<String>> {
    if NATIVE_MODULES.contains(&key) {
        return Some(vec!["std".to_string(), "native".to_string(), key.to_ascii_lowercase()]);
    }
    STD_PATHS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, p)| p.split('/').map(|s| s.to_string()).collect())
}

const DECORATORS: &[Item] = &[
    Item { label: "@Get", kind: Kind::Decorator, detail: "@Get(path: String)", doc: "Registers a GET route on the next `fn` for the real server.", insert: "@Get(\"/\")" },
    Item { label: "@Post", kind: Kind::Decorator, detail: "@Post(path: String)", doc: "Registers a POST route (valid JSON body → 201 echo; invalid → 400).", insert: "@Post(\"/\")" },
    Item { label: "@Put", kind: Kind::Decorator, detail: "@Put(path: String)", doc: "Registers a PUT route.", insert: "@Put(\"/\")" },
    Item { label: "@Delete", kind: Kind::Decorator, detail: "@Delete(path: String)", doc: "Registers a DELETE route.", insert: "@Delete(\"/\")" },
    Item { label: "@Test", kind: Kind::Decorator, detail: "@Test", doc: "Marks a function for `lex test` discovery.", insert: "@Test" },
    Item { label: "@Export", kind: Kind::Decorator, detail: "@Export", doc: "Marks a WASM plugin entry point.", insert: "@Export" },
];

// Exact lexer keyword surface (`lexicon-lexer/src/tokens.rs`; `nil`/`use`/
// `move`/`crate`/`typeof`/`Self` are NOT keywords — `null` is).
const KEYWORDS: &[&str] = &[
    "actor", "as", "async", "await", "break", "case", "catch", "channel",
    "class", "const", "constructor", "continue", "default", "defer", "derive",
    "do", "dynamic", "effect", "else", "enum", "extends", "extern", "false",
    "finally", "fn", "for", "function", "if", "implements", "import", "in",
    "inline", "interface", "is", "lambda", "let", "loop", "macro", "match",
    "module", "mut", "native", "new", "None", "null", "panic", "private",
    "protected", "pub", "recover", "ref", "return", "select", "self", "Some",
    "spawn", "static", "struct", "super", "switch", "task", "throw", "trait",
    "true", "try", "type", "unsafe", "var", "where", "while", "with", "yield",
];

const SNIPPETS: &[Item] = &[
    Item { label: "fn main", kind: Kind::Snippet, detail: "pub fn main() -> void { … }", doc: "Program entry point.", insert: "pub fn main() -> void {\n    $0\n}" },
    Item { label: "fn", kind: Kind::Snippet, detail: "pub fn name() -> T { … }", doc: "Function declaration.", insert: "pub fn ${1:name}() -> ${2:void} {\n    $0\n}" },
    Item { label: "match", kind: Kind::Snippet, detail: "match x { … }", doc: "Exhaustive pattern match (E0204 on missing arms).", insert: "match ${1:x} {\n    $0\n}" },
    Item { label: "struct", kind: Kind::Snippet, detail: "struct Name { … }", doc: "Struct declaration (`;` or `,` separators).", insert: "struct ${1:Name} {\n    $0\n}" },
    Item { label: "@Get fn", kind: Kind::Snippet, detail: "@Get + handler + serve", doc: "Minimal HTTP endpoint served by the real server.", insert: "@Get(\"${1:/}\")\npub fn ${2:handler}() -> String {\n    return \"${3:ok}\";\n}\n\npub fn main() -> void {\n    Http::serve(\"0.0.0.0:3000\");\n}" },
    Item { label: "Http::serve main", kind: Kind::Snippet, detail: "main serving an app", doc: "Entry point that blocks serving HTTP.", insert: "pub fn main() -> void {\n    Http::serve(\"0.0.0.0:${1:3000}\");\n}" },
    Item { label: "@Test fn", kind: Kind::Snippet, detail: "@Test fn …", doc: "Test discovered by `lex test`.", insert: "@Test\npub fn ${1:test_}() -> bool {\n    return $0;\n}" },
    Item { label: "import std", kind: Kind::Snippet, detail: "import std::<package>;", doc: "Stdlib import (key = last segment).", insert: "import std::${1:strings};" },
    Item { label: "lambda", kind: Kind::Snippet, detail: "|x| expr", doc: "First-class lambda (captures environment).", insert: "|${1:x}| ${2:x}" },
    Item { label: "assert", kind: Kind::Snippet, detail: "assert(cond, msg)", doc: "Runtime assertion (fails the run with msg).", insert: "assert(${1:cond}, \"${2:msg}\");" },
    Item { label: "let", kind: Kind::Snippet, detail: "let name = value;", doc: "Immutable binding (also `var`, `const`).", insert: "let ${1:name} = ${2:value};" },
    Item { label: "for", kind: Kind::Snippet, detail: "for x in xs { … }", doc: "Iteration (also `while`, `loop`).", insert: "for ${1:x} in ${2:xs} {\n    $0\n}" },
];

fn module_by_name(name: &str) -> Option<&'static Module> {
    MODULES.iter().find(|m| m.name == name)
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Complete a prefix typed by the user: `Http::s`, `Http::`, `@G`, `mat`, …
pub fn complete(prefix: &str) -> Vec<&'static Item> {
    let p = prefix.trim();
    if p.starts_with('@') {
        let needle = &p[1..];
        return DECORATORS.iter().filter(|d| d.label[1..].starts_with(needle)).collect();
    }
    // Member access: `Module::tail` or `Module.tail` (dot only when the
    // head is a known module, so `foo.bar` never fakes Http members).
    if let Some(end) = p.rfind("::") {
        let (head, needle) = (&p[..end], &p[end + 2..]);
        return match module_by_name(head) {
            Some(m) => m.members.iter().filter(|it| it.label.starts_with(needle)).collect(),
            None => Vec::new(),
        };
    }
    if let Some(end) = p.rfind('.') {
        let (head, needle) = (&p[..end], &p[end + 1..]);
        if let Some(m) = module_by_name(head) {
            return m.members.iter().filter(|it| it.label.starts_with(needle)).collect();
        }
        // Unknown `x.y` — no member completions.
        if !needle.is_empty() || !head.is_empty() {
            return Vec::new();
        }
    }
    // Bare word: modules + keywords + snippets.
    let mut out: Vec<&'static Item> = Vec::new();
    // Modules are materialised as Items on the fly — keep them in a static.
    for m in MODULES {
        if m.name.starts_with(p) {
            out.push(module_as_item(m));
        }
    }
    for k in KEYWORDS {
        if k.starts_with(p) {
            out.push(keyword_as_item(k));
        }
    }
    for s in SNIPPETS {
        if s.label.starts_with(p) {
            out.push(s);
        }
    }
    out.truncate(64);
    out
}

// Static views so bare-word completion can return `&'static Item`.
// MUST mirror MODULES 1:1 (`module_as_item` unwraps).
macro_rules! mi {
    ($label:literal, $detail:literal, $doc:literal) => {
        Item { label: $label, kind: Kind::Module, detail: $detail, doc: $doc, insert: concat!($label, "::") }
    };
}

const MODULE_ITEMS: &[Item] = &[
    mi!("Console", "module Console", "Stdout printing (native, no import)."),
    mi!("Json", "module Json", "Runtime JSON codec (native, no import)."),
    mi!("Env", "module Env", "Environment variables (native, no import)."),
    mi!("Http", "module Http", "Real HTTP server + blocking fetch (native, no import)."),
    mi!("Time", "module Time", "Wall clock (native, no import; see also `std::time`)."),
    mi!("File", "module File", "Filesystem (native, no import; see also `std::os`)."),
    mi!("Process", "module Process", "Args/exit/spawn (native, no import; see also `std::os::exec`)."),
    mi!("Text", "module Text", "Char-level string primitives (native, no import)."),
    mi!("Math", "module Math", "Exact float64 math (native, no import; see also `std::math`)."),
    mi!("List", "module List", "Native list algorithms (no import)."),
    mi!("Hash", "module Hash", "Native one-shot hashes (no import)."),
    mi!("Rand", "module Rand", "Native PRNG (no import; see also `std::rand`)."),
    mi!("Atomic", "module Atomic", "Shared atomic-i64 cells (native; see also `std::sync::atomic`)."),
    mi!("strings", "module strings", "`import std::strings;` — Go-parity strings."),
    mi!("strconv", "module strconv", "`import std::strconv;` — conversions."),
    mi!("math", "module math", "`import std::math;` — float64 math + consts."),
    mi!("sort", "module sort", "`import std::sort;` — native sorts + search."),
    mi!("slices", "module slices", "`import std::slices;` — list utilities + lambdas."),
    mi!("errors", "module errors", "`import std::errors;` — error values."),
    mi!("path", "module path", "`import std::path;` — slash paths."),
    mi!("fmt", "module fmt", "`import std::fmt;` — formatted I/O."),
    mi!("time", "module time", "`import std::time;` — wall clock + Format/Parse."),
    mi!("bytes", "module bytes", "`import std::bytes;` — byte lists."),
    mi!("io", "module io", "`import std::io;` — value-semantics streams."),
    mi!("bufio", "module bufio", "`import std::bufio;` — buffered reading + Scanner."),
    mi!("maps", "module maps", "`import std::maps;` — pair-list maps."),
    mi!("cmp", "module cmp", "`import std::cmp;` — three-way comparisons."),
    mi!("iter", "module iter", "`import std::iter;` — adapters over lambdas."),
    mi!("utf8", "module utf8", "`import std::unicode::utf8;` — runes."),
    mi!("list", "module list", "`import std::container::list;` — list vocabulary."),
    mi!("heap", "module heap", "`import std::container::heap;` — heap with lambda `less`."),
    mi!("ring", "module ring", "`import std::container::ring;` — circular ring."),
    mi!("os", "module os", "`import std::os;` — args/env/files."),
    mi!("exec", "module exec", "`import std::os::exec;` — subprocesses."),
    mi!("sync", "module sync", "`import std::sync;` — Mutex/WaitGroup/Once."),
    mi!("atomic", "module atomic", "`import std::sync::atomic;` — atomic int64 cells."),
    mi!("context", "module context", "`import std::context;` — cancel/timeout/values."),
    mi!("log", "module log", "`import std::log;` — default logger."),
    mi!("filepath", "module filepath", "`import std::path::filepath;` — OS paths + Walk/glob."),
    mi!("json", "module json", "`import std::encoding::json;` — Marshal/Unmarshal/Valid."),
    mi!("base64", "module base64", "`import std::encoding::base64;` — Std/URL codecs."),
    mi!("base32", "module base32", "`import std::encoding::base32;` — base32 codecs."),
    mi!("hex", "module hex", "`import std::encoding::hex;` — hex codec."),
    mi!("csv", "module csv", "`import std::encoding::csv;` — RFC-4180."),
    mi!("html", "module html", "`import std::html;` — escaping."),
    mi!("regexp", "module regexp", "`import std::regexp;` — documented-subset regex."),
    mi!("fnv", "module fnv", "`import std::hash::fnv;` — FNV-1a."),
    mi!("crc32", "module crc32", "`import std::hash::crc32;` — IEEE CRC-32."),
    mi!("adler32", "module adler32", "`import std::hash::adler32;` — Adler-32."),
    mi!("rand", "module rand", "`import std::rand;` — PRNG + shuffle."),
    mi!("subtle", "module subtle", "`import std::crypto::subtle;` — constant-time helpers."),
    mi!("sha256", "module sha256", "`import std::crypto::sha256;` — FIPS vectors."),
    mi!("hmac", "module hmac", "`import std::crypto::hmac;` — HMAC-SHA-256."),
    mi!("slog", "module slog", "`import std::slog;` — structured logging."),
    mi!("flag", "module flag", "`import std::flag;` — CLI flags."),
    mi!("mime", "module mime", "`import std::mime;` — media types."),
    mi!("url", "module url", "`import std::net::url;` — URL parse + query codec."),
    mi!("bits", "module bits", "`import std::math::bits;` — bit counting."),
    mi!("unsafe", "module unsafe", "`import std::unsafe;` — safe subset."),
    mi!("runtime", "module runtime", "`import std::runtime;` — GOOS/NumCPU/Version."),
    mi!("testing", "module testing", "`import std::testing;` — T/assertions/bench."),
    mi!("mail", "module mail", "`import std::net::mail;` — addresses."),
    mi!("textproto", "module textproto", "`import std::net::textproto;` — headers."),
    mi!("multipart", "module multipart", "`import std::mime::multipart;` — parts."),
    mi!("pem", "module pem", "`import std::encoding::pem;` — PEM blocks."),
    mi!("ascii85", "module ascii85", "`import std::encoding::ascii85;` — ascii85."),
    mi!("color", "module color", "`import std::image::color;` — colors."),
    mi!("tar", "module tar", "`import std::archive::tar;` — ustar."),
    mi!("crc64", "module crc64", "`import std::hash::crc64;` — ECMA CRC-64."),
    mi!("maphash", "module maphash", "`import std::hash::maphash;` — seeded hashes."),
    mi!("signal", "module signal", "`import std::os::signal;` — constants/Notify."),
    mi!("user", "module user", "`import std::os::user;` — current user."),
    mi!("expvar", "module expvar", "`import std::expvar;` — exported vars."),
    mi!("utf16", "module utf16", "`import std::unicode::utf16;` — surrogates."),
    mi!("Sys", "module Sys", "Host facts (native; see also `std::runtime`)."),
];

fn module_as_item(m: &'static Module) -> &'static Item {
    MODULE_ITEMS.iter().find(|i| i.label == m.name).unwrap()
}

fn keyword_as_item(k: &'static str) -> &'static Item {
    keyword_table().iter().find(|i| i.label == k).unwrap()
}

macro_rules! kw {
    ($label:literal, $doc:literal) => {
        Item { label: $label, kind: Kind::Keyword, detail: "keyword", doc: $doc, insert: $label }
    };
}

fn keyword_table() -> &'static [Item] {
    const T: &[Item] = &[
        kw!("actor", "actor declaration"),
        kw!("as", "explicit cast"),
        kw!("async", "async fn"),
        kw!("await", "await future"),
        kw!("break", "break loop"),
        kw!("case", "switch case"),
        kw!("catch", "catch clause"),
        kw!("channel", "channel type"),
        kw!("class", "class declaration"),
        kw!("const", "constant"),
        kw!("constructor", "constructor"),
        kw!("continue", "next iteration"),
        kw!("default", "default arm"),
        kw!("defer", "deferred call"),
        kw!("derive", "derive attribute"),
        kw!("do", "do block"),
        kw!("dynamic", "dynamic type"),
        kw!("effect", "effect declaration"),
        kw!("else", "else branch"),
        kw!("enum", "sum type"),
        kw!("extends", "inheritance"),
        kw!("extern", "extern block"),
        kw!("false", "boolean"),
        kw!("finally", "finally block"),
        kw!("fn", "function declaration"),
        kw!("for", "for loop"),
        kw!("function", "function item"),
        kw!("if", "conditional"),
        kw!("implements", "interface impl"),
        kw!("import", "import path"),
        kw!("in", "for-in membership"),
        kw!("inline", "inline hint"),
        kw!("interface", "interface declaration"),
        kw!("is", "type test"),
        kw!("lambda", "lambda expression"),
        kw!("let", "immutable binding"),
        kw!("loop", "infinite loop"),
        kw!("macro", "macro item"),
        kw!("match", "exhaustive match"),
        kw!("module", "module declaration"),
        kw!("mut", "mutable binding"),
        kw!("native", "native item"),
        kw!("new", "allocation / struct construction"),
        kw!("None", "option none"),
        kw!("null", "null value"),
        kw!("panic", "panic call"),
        kw!("private", "private visibility"),
        kw!("protected", "protected visibility"),
        kw!("pub", "public visibility"),
        kw!("recover", "recover call"),
        kw!("ref", "reference pattern"),
        kw!("return", "return value"),
        kw!("select", "select over channels"),
        kw!("self", "receiver"),
        kw!("Some", "option some"),
        kw!("spawn", "spawns a task"),
        kw!("static", "static item"),
        kw!("struct", "struct declaration"),
        kw!("super", "parent module"),
        kw!("switch", "switch statement"),
        kw!("task", "task item"),
        kw!("throw", "throw (desugars to panic)"),
        kw!("trait", "trait declaration"),
        kw!("true", "boolean"),
        kw!("try", "try expression"),
        kw!("type", "type alias"),
        kw!("unsafe", "unsafe block"),
        kw!("var", "mutable binding"),
        kw!("where", "generic bounds"),
        kw!("while", "while loop"),
        kw!("with", "with expression"),
        kw!("yield", "yield value"),
    ];
    T
}

/// Hover: `Http::serve` → (signature, doc); `Http::` → module doc.
pub fn hover(word: &str) -> Option<(&'static str, &'static str)> {
    let mut w = word.trim().trim_end_matches(':');
    let is_decorator = w.starts_with('@');
    if is_decorator {
        w = &w[1..];
        return DECORATORS.iter().find(|d| &d.label[1..] == w).map(|d| (d.detail, d.doc));
    }
    if let Some(idx) = w.find("::") {
        let (head, tail) = (&w[..idx], &w[idx + 2..]);
        let m = module_by_name(head)?;
        if tail.is_empty() {
            let item = module_as_item(m);
            return Some((item.detail, item.doc));
        }
        if let Some(it) = m.members.iter().find(|it| it.label == tail) {
            return Some((it.detail, it.doc));
        }
        // Mid-typing hover (`Http::se`): show the member when the partial
        // tail matches exactly one candidate.
        let mut cand = m.members.iter().filter(|it| it.label.starts_with(tail));
        match (cand.next(), cand.next()) {
            (Some(only), None) => return Some((only.detail, only.doc)),
            _ => return None,
        }
    }
    if let Some(m) = module_by_name(w) {
        let item = module_as_item(m);
        return Some((item.detail, item.doc));
    }
    None
}

/// Extract the completion prefix from a line up to a cursor column.
/// E.g. `    Http::se█` → `Http::se`.
pub fn prefix_at(line: &str, col: usize) -> String {
    let upto: String = line.chars().take(col).collect();
    let bytes = upto.as_bytes();
    let mut start = bytes.len();
    while start > 0 {
        let c = bytes[start - 1] as char;
        if c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '.' || c == '@' {
            start -= 1;
        } else {
            break;
        }
    }
    let token = &upto[start..];
    // No completions inside numeric literals (`1`, `3.5`).
    if token.starts_with(|c: char| c.is_ascii_digit()) {
        return String::new();
    }
    token.to_string()
}

/// `lex complete` entry point.
pub fn complete_cmd(
    prefix: Option<String>,
    file: Option<String>,
    line: Option<usize>,
    col: Option<usize>,
    json: bool,
) -> Result<()> {
    let p = if let Some(p) = prefix {
        p
    } else if let (Some(f), Some(l), Some(c)) = (file, line, col) {
        let src = std::fs::read_to_string(&f)?;
        let text = src.lines().nth(l.saturating_sub(1)).unwrap_or("");
        prefix_at(text, c)
    } else {
        anyhow::bail!("usage: lex complete --prefix <p> | --file <f> --line <n> --col <c>");
    };
    let items = complete(&p);
    if json {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|it| {
                serde_json::json!({
                    "label": it.label,
                    "kind": it.kind.as_str(),
                    "detail": it.detail,
                    "doc": it.doc,
                    "insert": it.insert,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
    } else {
        for it in &items {
            println!("{}\t{}\t{}", it.label, it.kind.as_str(), it.detail);
        }
        if items.is_empty() {
            println!("(no completions for {:?})", p);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_member_completion() {
        let labels: Vec<_> = complete("Http::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"serve"));
        assert!(labels.contains(&"get"));
        assert!(labels.contains(&"post"));
    }

    #[test]
    fn http_prefix_filters() {
        let labels: Vec<_> = complete("Http::s").iter().map(|i| i.label).collect();
        assert_eq!(labels, vec!["serve"]);
    }

    #[test]
    fn dot_form_also_completes_members() {
        let labels: Vec<_> = complete("Console.").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"writeLine"));
    }

    #[test]
    fn decorator_completion() {
        let labels: Vec<_> = complete("@G").iter().map(|i| i.label).collect();
        assert_eq!(labels, vec!["@Get"]);
    }

    #[test]
    fn bare_word_suggests_modules_and_keywords() {
        let labels: Vec<_> = complete("Ht").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"Http"));
        assert!(complete("").len() > 10);
    }

    #[test]
    fn unknown_module_is_empty() {
        assert!(complete("Nope::").is_empty());
    }

    #[test]
    fn hover_resolves() {
        let (sig, _) = hover("Http::serve").expect("serve hover");
        assert!(sig.contains("serve"));
        let (msig, _) = hover("Http::").expect("module hover");
        assert!(msig.contains("module"));
        // Mid-typing: unique partial tail resolves.
        let (psig, _) = hover("Http::se").expect("partial hover");
        assert!(psig.contains("serve"));
        assert!(hover("Nope::x").is_none());
    }

    #[test]
    fn prefix_extraction() {
        assert_eq!(prefix_at("    Http::se", 12), "Http::se");
        assert_eq!(prefix_at("@G", 2), "@G");
        assert_eq!(prefix_at("let x = 1", 9), "");
    }

    #[test]
    fn std_package_completion() {
        let labels: Vec<_> = complete("strings::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"ToUpper"));
        assert!(labels.contains(&"Cut"));
        let labels: Vec<_> = complete("strings::ToU").iter().map(|i| i.label).collect();
        assert_eq!(labels, vec!["ToUpper"]);
        // Nested-path keys complete under the last segment.
        let labels: Vec<_> = complete("utf8::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"RuneCount"));
        let labels: Vec<_> = complete("sha256::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"Sum"));
        let labels: Vec<_> = complete("exec::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"Output"));
    }

    #[test]
    fn native_builtin_completion() {
        let labels: Vec<_> = complete("Atomic::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"make"));
        assert!(labels.contains(&"cas"));
        let labels: Vec<_> = complete("Math::s").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"sin"));
        assert!(labels.contains(&"sqrt"));
        let labels: Vec<_> = complete("Json::").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"parse"));
        assert!(labels.contains(&"valid"));
        // Removed fictional entries stay gone.
        assert!(complete("Crypto::").is_empty());
        assert!(complete("Blob::").is_empty());
    }

    #[test]
    fn keywords_match_the_lexer() {
        let labels: Vec<_> = complete("le").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"let"));
        assert!(complete("null").iter().any(|i| i.label == "null"));
        assert!(!complete("nil").iter().any(|i| i.label == "nil"));
        assert!(complete("spawn").iter().any(|i| i.label == "spawn"));
    }

    #[test]
    fn std_snippets_present() {
        let labels: Vec<_> = complete("imp").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"import std"));
        let labels: Vec<_> = complete("lamb").iter().map(|i| i.label).collect();
        assert!(labels.contains(&"lambda"));
    }

    #[test]
    fn sdk_path_map_covers_definition() {
        assert_eq!(
            module_sdk_path("strings"),
            Some(vec!["std".to_string(), "strings".to_string()])
        );
        assert_eq!(
            module_sdk_path("list"),
            Some(vec!["std".to_string(), "container".to_string(), "list".to_string()])
        );
        assert_eq!(
            module_sdk_path("Math"),
            Some(vec!["std".to_string(), "native".to_string(), "math".to_string()])
        );
        assert_eq!(module_sdk_path("Nope"), None);
    }

    #[test]
    fn third_wave_modules_complete() {        for (prefix, want) in [
            ("Sys::", "ncpu"),
            ("unsafe::", "Sizeof"),
            ("tar::", "Next"),
            ("crc64::", "Checksum"),
            ("url::", "QueryEscape"),
        ] {
            let labels: Vec<_> = complete(prefix).iter().map(|i| i.label).collect();
            assert!(labels.contains(&want), "{}{}", prefix, want);
        }
    }

    #[test]
    fn hover_covers_std_and_natives() {        let (sig, _) = hover("strings::ToUpper").expect("std hover");
        assert!(sig.contains("ToUpper"));
        let (sig, _) = hover("Atomic::cas").expect("native hover");
        assert!(sig.contains("cas"));
        let (sig, _) = hover("sha256::").expect("std module hover");
        assert!(sig.contains("module"));
    }
}
