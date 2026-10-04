// Lexicon Standard Library — crypto/rc4.
// Go-parity RC4 stream cipher (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// One-shot XOR over the generated keystream; keystream alone is exposed for
// callers that need to resume. RC4 is deprecated (biases in the first bytes
// and in TLS): keep it for legacy protocols and checksums only.
// Import as `import std::crypto::rc4;`.

/// Keystream of `n` bytes for `key` (Go's `rc4.NewCipher` + Read).
pub fn Keystream(key: String, n: i64) -> Dynamic {
    let s = ksa(key);
    let out = [];
    let i = 0;
    let j = 0;
    let k = 0;
    while k < n {
        i = (i + 1) % 256;
        j = (j + s[i]) % 256;
        let t = s[i];
        s[i] = s[j];
        s[j] = t;
        out.push(s[(s[i] + s[j]) % 256]);
        k = k + 1;
    }
    return out;
}

/// RC4-encrypted bytes (Go: cipher over the same keystream; XOR is its own
/// inverse, so this decrypts as well).
pub fn Cipher(key: String, data: String) -> Dynamic {
    let ks = Keystream(key, data.len());
    let out = [];
    let i = 0;
    while i < data.len() {
        out.push(Text::code_at(data, i) ^ ks[i]);
        i = i + 1;
    }
    return out;
}

/// Uppercase hex of the cipher output (the classic test-vector shape).
pub fn Hex(key: String, data: String) -> String {
    return hexof(Cipher(key, data), "0123456789ABCDEF");
}

/// Lowercase hex of the cipher output.
pub fn HexLower(key: String, data: String) -> String {
    return hexof(Cipher(key, data), "0123456789abcdef");
}

// Key-scheduling algorithm: identity permutation stirred with the key.
fn ksa(key: String) -> Dynamic {
    let s = [];
    let i = 0;
    while i < 256 {
        s.push(i);
        i = i + 1;
    }
    let j = 0;
    i = 0;
    let klen = key.len();
    while i < 256 {
        j = (j + s[i] + Text::code_at(key, i % klen)) % 256;
        let t = s[i];
        s[i] = s[j];
        s[j] = t;
        i = i + 1;
    }
    return s;
}

fn hexof(b: Dynamic, digits: String) -> String {
    let out = "";
    let i = 0;
    while i < b.len() {
        out = out + Text::slice(digits, b[i] / 16, b[i] / 16 + 1);
        out = out + Text::slice(digits, b[i] % 16, b[i] % 16 + 1);
        i = i + 1;
    }
    return out;
}
