// Lexicon Standard Library — crypto/sha1.
// Go-parity SHA-1 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6, FIPS 180-4).
// Pure Lex compression, same exact-i64 idiom as `crypto/sha256` (rotate via
// the POW2 table, mask with `% 4294967296`). SHA-1 is broken for signatures:
// use it for checksums and legacy protocols only, never for security.
// Import as `import std::crypto::sha1;`.

const sha1_K = [1518500249, 1859775393, 2400959708, 3395469782];

// 2^0 .. 2^32 (rotate indexes here instead of looping per call).
const POW2 = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, 2147483648, 4294967296];

const M32 = 4294967295;
const MOD32 = 4294967296;

/// 20 digest bytes as a list (Go's `sha1.Sum`).
pub fn SumBytes(s: String) -> Dynamic {
    let msg = [];
    let i = 0;
    while i < s.len() {
        msg.push(Text::code_at(s, i));
        i = i + 1;
    }
    let bitlen = msg.len() * 8;
    msg.push(128);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    // 64-bit big-endian bit length: high word then low word.
    let hi = bitlen / MOD32;
    let lo = bitlen % MOD32;
    msg.push((hi / 16777216) % 256);
    msg.push((hi / 65536) % 256);
    msg.push((hi / 256) % 256);
    msg.push(hi % 256);
    msg.push((lo / 16777216) % 256);
    msg.push((lo / 65536) % 256);
    msg.push((lo / 256) % 256);
    msg.push(lo % 256);
    let h0 = 1732584193;
    let h1 = 4023233417;
    let h2 = 2562383102;
    let h3 = 271733878;
    let h4 = 3285377520;
    let off = 0;
    while off < msg.len() {
        let w = [];
        let t = 0;
        while t < 16 {
            let p = off + t * 4;
            w.push((msg[p] * 16777216 + msg[p + 1] * 65536 + msg[p + 2] * 256 + msg[p + 3]) % MOD32);
            t = t + 1;
        }
        while t < 80 {
            let v = w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16];
            w.push(rotl(v, 1));
            t = t + 1;
        }
        let a = h0;
        let b = h1;
        let c = h2;
        let d = h3;
        let e = h4;
        t = 0;
        while t < 80 {
            let f = 0;
            let k = 0;
            if t < 20 {
                f = bitor(b & c, (M32 - b) & d);
                k = sha1_K[0];
            }
            if t >= 20 && t < 40 {
                f = b ^ c ^ d;
                k = sha1_K[1];
            }
            if t >= 40 && t < 60 {
                f = bitor(bitor(b & c, b & d), c & d);
                k = sha1_K[2];
            }
            if t >= 60 {
                f = b ^ c ^ d;
                k = sha1_K[3];
            }
            let tmp = (rotl(a, 5) + f + e + k + w[t]) % MOD32;
            e = d;
            d = c;
            c = rotl(b, 30);
            b = a;
            a = tmp;
            t = t + 1;
        }
        h0 = (h0 + a) % MOD32;
        h1 = (h1 + b) % MOD32;
        h2 = (h2 + c) % MOD32;
        h3 = (h3 + d) % MOD32;
        h4 = (h4 + e) % MOD32;
        off = off + 64;
    }
    return be_bytes([h0, h1, h2, h3, h4]);
}

/// Hex digest, lowercase (Go's `hex.EncodeToString(sha1.New().Sum(nil))`).
pub fn Sum(s: String) -> String {
    return HexOf(SumBytes(s));
}

/// Hex-encodes a byte list (same shape as `crypto/sha256.HexOf`).
pub fn HexOf(b: Dynamic) -> String {
    let digits = "0123456789abcdef";
    let out = "";
    let i = 0;
    while i < b.len() {
        out = out + Text::slice(digits, b[i] / 16, b[i] / 16 + 1);
        out = out + Text::slice(digits, b[i] % 16, b[i] % 16 + 1);
        i = i + 1;
    }
    return out;
}

// Bitwise OR without a `|` operator: a|b == a + b - (a&b) for disjoint bits.
fn bitor(a: i64, b: i64) -> i64 {
    return a + b - (a & b);
}

// 32-bit rotate right via table lookup (exact: operands stay below 2^32).
fn rotr(x: i64, n: i64) -> i64 {
    let p = POW2[n];
    let lo = x % p;
    let hi = (x - lo) / p;
    return (lo * POW2[32 - n] + hi) % MOD32;
}

fn rotl(x: i64, n: i64) -> i64 {
    return rotr(x, 32 - n);
}

fn be_bytes(ws: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < ws.len() {
        let w = ws[i];
        out.push(w / 16777216);
        out.push((w / 65536) % 256);
        out.push((w / 256) % 256);
        out.push(w % 256);
        i = i + 1;
    }
    return out;
}
