// Lexicon SDK — native signature stubs (Text).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Code point at char offset `i` (-1 when out of range).
pub fn code_at(s: String, i: i64) -> i64 {
    panic("native stub");
}

/// One-character string for a code point.
pub fn from_code(cp: i64) -> String {
    panic("native stub");
}

/// Character (not byte) count.
pub fn len(s: String) -> i64 {
    panic("native stub");
}

/// Char-offset `[lo:hi)` slice, clamped.
pub fn slice(s: String, lo: i64, hi: i64) -> String {
    panic("native stub");
}

/// First char offset of `sub`, or -1.
pub fn index_of(s: String, sub: String) -> i64 {
    panic("native stub");
}

/// Last char offset of `sub`, or -1.
pub fn last_index_of(s: String, sub: String) -> i64 {
    panic("native stub");
}

/// Prefix test.
pub fn starts_with(s: String, prefix: String) -> bool {
    panic("native stub");
}

/// Suffix test.
pub fn ends_with(s: String, suffix: String) -> bool {
    panic("native stub");
}

/// `n` copies concatenated.
pub fn repeat(s: String, n: i64) -> String {
    panic("native stub");
}

/// Joins list elements with `sep`.
pub fn join(list: Dynamic, sep: String) -> String {
    panic("native stub");
}
