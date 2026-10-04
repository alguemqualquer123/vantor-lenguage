// Lexicon Standard Library — hash/maphash.
// Go-parity seeded hashing (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// Go's `maphash` randomizes per process; here the seed travels
// explicitly (hashes cross function boundaries as values), backed by the
// native seeded FNV-1a64. Import as `import std::hash::maphash;`.

/// A hash seed (Go's `maphash.Seed`).
pub struct Seed {
    s: i64,
}

/// Makes a seed from any integer (Go's `maphash.MakeSeed`; wall-clock
/// seeds come from `rand::Int`).
pub fn MakeSeed(n: i64) -> Seed {
    return Seed { s: n };
}

/// Hashes a string under `seed` (Go's `maphash.String`).
pub fn String(seed: Seed, s: String) -> i64 {
    return Hash::maphash(seed.s, s);
}

/// Hashes bytes under `seed` (Go's `maphash.Bytes`, list-adapted).
pub fn Bytes(seed: Seed, b: Dynamic) -> i64 {
    let s = "";
    let i = 0;
    while i < b.len() {
        s = s + Text::from_code(b[i]);
        i = i + 1;
    }
    return Hash::maphash(seed.s, s);
}

/// Hashes an integer under `seed` (Go's `maphash comparable` path for
/// ints: seed-mixed splitmix64, bits-wrapped, exact logical shifts).
pub fn Int(seed: Seed, v: i64) -> i64 {
    let z = (v + seed.s) * -7046029288634856825;
    z = (z ^ lshr(z, 26)) * -7046029288634856825;
    return z ^ lshr(z, 34);
}

fn lshr(x: i64, n: i64) -> i64 {
    if n <= 0 {
        return x;
    }
    if n >= 64 {
        return 0;
    }
    if x >= 0 {
        return x / pow2i(n);
    }
    // Truncation rounds toward zero; logical shift needs floor, hence
    // the -1 correction when there is a remainder (all wrapping-safe).
    let p = pow2i(n);
    let t = x / p;
    if x % p == 0 {
        return t + pow2i(64 - n);
    }
    return t - 1 + pow2i(64 - n);
}

fn pow2i(n: i64) -> i64 {
    if n >= 63 {
        return 0 - 9223372036854775807 - 1;
    }
    let p = 1;
    let i = 0;
    while i < n {
        p = p * 2;
        i = i + 1;
    }
    return p;
}
