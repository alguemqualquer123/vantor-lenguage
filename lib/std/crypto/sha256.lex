// Lexicon Standard Library — crypto/sha256.
// Go-parity SHA-256 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6). Pure Lex
// FIPS 180-4 compression over exact i64 arithmetic (all sums reduced mod
// 2^32 every round). ASCII fast path like the other byte codecs.
// Import as `import std::crypto::sha256;`.

const sha_K = [1116352408, 1899447441, 3049323471, 3921009573, 961987163, 1508970993, 2453635748, 2870763221, 3624381080, 310598401, 607225278, 1426881987, 1925078388, 2162078206, 2614888103, 3248222580, 3835390401, 4022224774, 264347078, 604807628, 770255983, 1249150122, 1555081692, 1996064986, 2554220882, 2821834349, 2952996808, 3210313671, 3336571891, 3584528711, 113926993, 338241895, 666307205, 773529912, 1294757372, 1396182291, 1695183700, 1986661051, 2177026350, 2456956037, 2730485921, 2820302411, 3259730800, 3345764771, 3516065817, 3600352804, 4094571909, 275423344, 430227734, 506948616, 659060556, 883997877, 958139571, 1322822218, 1537002063, 1747873779, 1955562222, 2024104815, 2227730452, 2361852424, 2428436474, 2756734187, 3204031479, 3329325298];

// 2^0 .. 2^32 (rotr indexes here instead of looping `pow2n` per call:
// that helper alone cost ~32M interpreter steps on a 20KB digest).
const POW2 = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, 2147483648, 4294967296];

/// 32 digest bytes as a list (Go's `sha256.Sum256`, list-adapted).
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
    msg.push(0);
    msg.push(0);
    msg.push(0);
    msg.push(0);
    msg.push((bitlen / 16777216) % 256);
    msg.push((bitlen / 65536) % 256);
    msg.push((bitlen / 256) % 256);
    msg.push(bitlen % 256);
    let h0 = 1779033703;
    let h1 = 3144134277;
    let h2 = 1013904242;
    let h3 = 2773480762;
    let h4 = 1359893119;
    let h5 = 2600822924;
    let h6 = 528734635;
    let h7 = 1541459225;
    let off = 0;
    while off < msg.len() {
        let w = [];
        let t = 0;
        while t < 16 {
            w.push((msg[off + t * 4] * 16777216 + msg[off + t * 4 + 1] * 65536 + msg[off + t * 4 + 2] * 256 + msg[off + t * 4 + 3]) % 4294967296);
            t = t + 1;
        }
        t = 16;
        while t < 64 {
            let s0 = (rotr(w[t - 15], 7) ^ rotr(w[t - 15], 18) ^ (w[t - 15] / 8)) % 4294967296;
            let s1 = (rotr(w[t - 2], 17) ^ rotr(w[t - 2], 19) ^ (w[t - 2] / 1024)) % 4294967296;
            w.push((w[t - 16] + s0 + w[t - 7] + s1) % 4294967296);
            t = t + 1;
        }
        let a = h0;
        let b = h1;
        let c = h2;
        let d = h3;
        let e = h4;
        let f = h5;
        let g = h6;
        let hh = h7;
        t = 0;
        while t < 64 {
            let S1 = (rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25)) % 4294967296;
            let ch = ((e & f) ^ ((4294967295 - e) & g)) % 4294967296;
            let t1 = (hh + S1 + ch + sha_K[t] + w[t]) % 4294967296;
            let S0 = (rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22)) % 4294967296;
            let maj = ((a & b) ^ (a & c) ^ (b & c)) % 4294967296;
            let t2 = (S0 + maj) % 4294967296;
            hh = g;
            g = f;
            f = e;
            e = (d + t1) % 4294967296;
            d = c;
            c = b;
            b = a;
            a = (t1 + t2) % 4294967296;
            t = t + 1;
        }
        h0 = (h0 + a) % 4294967296;
        h1 = (h1 + b) % 4294967296;
        h2 = (h2 + c) % 4294967296;
        h3 = (h3 + d) % 4294967296;
        h4 = (h4 + e) % 4294967296;
        h5 = (h5 + f) % 4294967296;
        h6 = (h6 + g) % 4294967296;
        h7 = (h7 + hh) % 4294967296;
        off = off + 64;
    }
    return words([h0, h1, h2, h3, h4, h5, h6, h7]);
}

/// Hex digest (native one-shot; the pure-Lex compression below stays as
/// the documented reference and backs `SumBytes`).
pub fn Sum(s: String) -> String {
    return Hash::sha256hex(s);
}

/// Hex-encodes a byte list (shared with `hmac`).
pub fn HexOf(b: Dynamic) -> String {
    return hexof(b);
}

fn words(hs: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < hs.len() {
        let wv = hs[i];
        out.push(wv / 16777216);
        out.push((wv / 65536) % 256);
        out.push((wv / 256) % 256);
        out.push(wv % 256);
        i = i + 1;
    }
    return out;
}

fn hexof(b: Dynamic) -> String {
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

// 32-bit rotate right via table lookup (exact: operands stay below 2^32).
fn rotr(x: i64, n: i64) -> i64 {
    let p = POW2[n];
    let lo = x % p;
    let hi = (x - lo) / p;
    return (lo * POW2[32 - n] + hi) % 4294967296;
}
