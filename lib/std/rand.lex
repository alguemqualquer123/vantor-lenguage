// Lexicon Standard Library — rand.
// Go-parity pseudo-random numbers (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 6 — `math/rand` shape). Entropy comes from the native xorshift64*
// `Rand::*` builtins (seeded from the wall clock); shuffling and ranges
// stay in Lex. Import as `import std::rand;`.

/// Seeds the generator (Go's `rand.Seed`; same sequence for the same seed).
pub fn Seed(seed: i64) -> void {
    Rand::seed(seed);
}

/// Returns a non-negative i64 (Go's `rand.Int`).
pub fn Int() -> i64 {
    let v = Rand::int();
    if v < 0 {
        return 0 - v;
    }
    return v;
}

/// Returns a value in [0, n) (Go's `rand.Intn`; panics when n <= 0).
pub fn Intn(n: i64) -> i64 {
    return Rand::intn(n);
}

/// Returns a float in [0, 1) (Go's `rand.Float64`).
pub fn Float64() -> f64 {
    return Rand::float();
}

/// Returns `n` random bytes as a list (Go's `rand.Read` shape).
pub fn Bytes(n: i64) -> Dynamic {
    return Rand::bytes(n);
}

/// Returns a value in [lo, hi) (convenience over `Intn`).
pub fn Range(lo: i64, hi: i64) -> i64 {
    return lo + Intn(hi - lo);
}

/// Returns a shuffled copy (Fisher-Yates, Go's `rand.Shuffle` adapted).
pub fn Shuffle(xs: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < xs.len() {
        out.push(xs[i]);
        i = i + 1;
    }
    let j = out.len() - 1;
    while j > 0 {
        let k = Intn(j + 1);
        let t = out[j];
        out[j] = out[k];
        out[k] = t;
        j = j - 1;
    }
    return out;
}

/// Picks one element uniformly (panics on empty lists, like Go's
/// out-of-range access).
pub fn Choice(xs: Dynamic) -> Dynamic {
    return xs[Intn(xs.len())];
}
