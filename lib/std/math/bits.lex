// Lexicon Standard Library — math/bits.
// Go-parity bit counting (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1/6).
// The interpreter wraps i64 arithmetic, so every op below is bit-exact;
// bit lists are extracted LSB-first from the non-negative low 63 bits
// plus an explicit sign bit (division truncates, so negatives never go
// through `/` directly). Import as `import std::math::bits;`.

/// Counts one bits in the 64-bit view (Go's `bits.OnesCount64`).
pub fn OnesCount(x: i64) -> i64 {
    let b = bits_of(x);
    let n = 0;
    let i = 0;
    while i < 64 {
        n = n + b[i];
        i = i + 1;
    }
    return n;
}

/// Counts trailing zero bits; 64 when `x == 0` (Go's `bits.TrailingZeros64`).
pub fn TrailingZeros(x: i64) -> i64 {
    if x == 0 {
        return 64;
    }
    // Halving an even number is exact under truncation, so this loop is
    // correct for negatives too.
    let n = 0;
    let v = x;
    while v % 2 == 0 {
        n = n + 1;
        v = v / 2;
    }
    return n;
}

/// Counts leading zero bits; 64 when `x == 0`, 0 when `x < 0`
/// (Go's `bits.LeadingZeros64`).
pub fn LeadingZeros(x: i64) -> i64 {
    if x == 0 {
        return 64;
    }
    if x < 0 {
        return 0;
    }
    let n = 1;
    let bit = 4611686018427387904;
    while bit > 0 {
        if (x / bit) % 2 != 0 {
            return n;
        }
        n = n + 1;
        bit = bit / 2;
    }
    return n;
}

/// Bit length: bits needed to represent `x`; 0 when `x == 0`, 64 when
/// `x < 0` (Go's `bits.Len64`).
pub fn Len(x: i64) -> i64 {
    if x == 0 {
        return 0;
    }
    if x < 0 {
        return 64;
    }
    return 64 - LeadingZeros(x);
}

/// Rotates left by `k` (Go's `bits.RotateLeft64`; `k` mod 64, negative
/// `k` rotates right).
pub fn RotateLeft(x: i64, k: i64) -> i64 {
    let s = k % 64;
    if s < 0 {
        s = s + 64;
    }
    if s == 0 {
        return x;
    }
    let b = bits_of(x);
    let out = 0;
    let j = 63;
    while j >= 0 {
        let src = j - s;
        if src < 0 {
            src = src + 64;
        }
        out = out * 2 + b[src];
        j = j - 1;
    }
    return out;
}

/// Reverses bit order (Go's `bits.Reverse64`).
pub fn Reverse(x: i64) -> i64 {
    let b = bits_of(x);
    let out = 0;
    let j = 0;
    while j < 64 {
        out = out * 2 + b[j];
        j = j + 1;
    }
    return out;
}

/// Reverses byte order (Go's `bits.ReverseBytes64`).
pub fn ReverseBytes(x: i64) -> i64 {
    let b = bits_of(x);
    let out = 0;
    let p = 63;
    while p >= 0 {
        let q = (7 - p / 8) * 8 + p % 8;
        out = out * 2 + b[q];
        p = p - 1;
    }
    return out;
}

// 64 bits LSB-first: low 63 via exact division, sign bit explicit.
fn bits_of(x: i64) -> Dynamic {
    let b = [];
    let v = x & 9223372036854775807;
    let i = 0;
    while i < 63 {
        b.push(v % 2);
        v = v / 2;
        i = i + 1;
    }
    if x < 0 {
        b.push(1);
    } else {
        b.push(0);
    }
    return b;
}
