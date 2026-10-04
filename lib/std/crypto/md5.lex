// Lexicon Standard Library — crypto/md5.
// Go-parity MD5 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6, RFC 1321).
// Pure Lex compression over exact i64 arithmetic, every sum masked to 32
// bits like `crypto/sha256`. There is no `|` or `~` operator in Lex, so
// OR is `a + b - (a & b)` and NOT is `4294967295 - a`. MD5 is a checksum,
// not a password hash: use `crypto/sha256` for anything security-facing.
// Import as `import std::crypto::md5;`.

const md5_K = [3614090360, 3905402710, 606105819, 3250441966, 4118548399, 1200080426, 2821735955, 4249261313, 1770035416, 2336552879, 4294925233, 2304563134, 1804603682, 4254626195, 2792965006, 1236535329, 4129170786, 3225465664, 643717713, 3921069994, 3593408605, 38016083, 3634488961, 3889429448, 568446438, 3275163606, 4107603335, 1163531501, 2850285829, 4243563512, 1735328473, 2368359562, 4294588738, 2272392833, 1839030562, 4259657740, 2763975236, 1272893353, 4139469664, 3200236656, 681279174, 3936430074, 3572445317, 76029189, 3654602809, 3873151461, 530742520, 3299628645, 4096336452, 1126891415, 2878612391, 4237533241, 1700485571, 2399980690, 4293915773, 2240044497, 1873313359, 4264355552, 2734768916, 1309151649, 4149444226, 3174756917, 718787259, 3951481745];

const md5_S = [7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21];

const POW2 = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, 2147483648, 4294967296];

const M32 = 4294967295;

/// 16 digest bytes as a list (Go's `md5.Sum`, list-adapted).
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
    // Length trailer is little-endian (the opposite of SHA-2).
    msg.push(bitlen % 256);
    msg.push((bitlen / 256) % 256);
    msg.push((bitlen / 65536) % 256);
    msg.push((bitlen / 16777216) % 256);
    msg.push((bitlen / 4294967296) % 256);
    msg.push((bitlen / 1099511627776) % 256);
    msg.push(0);
    msg.push(0);
    let a0 = 1732584193;
    let b0 = 4023233417;
    let c0 = 2562383102;
    let d0 = 271733878;
    let off = 0;
    while off < msg.len() {
        let m = [];
        let t = 0;
        while t < 16 {
            let p = off + t * 4;
            m.push((msg[p] + msg[p + 1] * 256 + msg[p + 2] * 65536 + msg[p + 3] * 16777216) % 4294967296);
            t = t + 1;
        }
        let a = a0;
        let b = b0;
        let c = c0;
        let d = d0;
        t = 0;
        while t < 64 {
            let nb = M32 - b;
            let nd = M32 - d;
            let f = 0;
            let g = 0;
            if t < 16 {
                f = bitor(b & c, nb & d);
                g = t;
            }
            if t >= 16 && t < 32 {
                f = bitor(d & b, nd & c);
                g = (5 * t + 1) % 16;
            }
            if t >= 32 && t < 48 {
                f = b ^ c ^ d;
                g = (3 * t + 5) % 16;
            }
            if t >= 48 {
                f = c ^ bitor(b, nd);
                g = (7 * t) % 16;
            }
            f = (f + a + md5_K[t] + m[g]) % 4294967296;
            let rot = rotl(f, md5_S[t]) % 4294967296;
            a = d;
            d = c;
            c = b;
            b = (b + rot) % 4294967296;
            t = t + 1;
        }
        a0 = (a0 + a) % 4294967296;
        b0 = (b0 + b) % 4294967296;
        c0 = (c0 + c) % 4294967296;
        d0 = (d0 + d) % 4294967296;
        off = off + 64;
    }
    return le_bytes([a0, b0, c0, d0]);
}

/// Hex digest, lowercase (Go's `fmt.Sprintf("%x", md5.Sum(b))`).
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
    return (lo * POW2[32 - n] + hi) % 4294967296;
}

fn rotl(x: i64, n: i64) -> i64 {
    return rotr(x, 32 - n);
}

fn le_bytes(ws: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    while i < ws.len() {
        let w = ws[i];
        out.push(w % 256);
        out.push((w / 256) % 256);
        out.push((w / 65536) % 256);
        out.push((w / 16777216) % 256);
        i = i + 1;
    }
    return out;
}
