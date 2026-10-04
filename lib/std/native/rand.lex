// Lexicon SDK — native signature stubs (Rand).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Uniform value in [0, n).
pub fn intn(n: i64) -> i64 {
    panic("native stub");
}

/// Random i64 (xorshift64*, wall-clock seeded).
pub fn int() -> i64 {
    panic("native stub");
}

/// Uniform float in [0, 1).
pub fn float() -> f64 {
    panic("native stub");
}

/// `n` random bytes.
pub fn bytes(n: i64) -> Dynamic {
    panic("native stub");
}

/// Reseeds the generator (deterministic replays).
pub fn seed(s: i64) -> void {
    panic("native stub");
}
