// Lexicon SDK — native signature stubs (Json).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Serializes a value to JSON.
pub fn serialize(value: Dynamic) -> String {
    panic("native stub");
}

/// Alias of `serialize` (same native builtin).
pub fn stringify(value: Dynamic) -> String {
    panic("native stub");
}

/// Parses JSON (aborts on invalid input — guard with `valid`).
pub fn parse(text: String) -> Dynamic {
    panic("native stub");
}

/// Reports whether the text is well-formed JSON (never aborts).
pub fn valid(text: String) -> bool {
    panic("native stub");
}
