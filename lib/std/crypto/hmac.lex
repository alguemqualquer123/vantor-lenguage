// Lexicon Standard Library — crypto/hmac.
// Go-parity HMAC (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6, RFC 2104).
// Keys/messages cross as strings (byte-valued code points 0-255 round-trip
// exactly through `Text::from_code`/`code_at`). Import as
// `import std::crypto::hmac;`.

import std::crypto::sha256;

/// HMAC-SHA-256 hex digest (Go's `hmac.New(sha256.New, key)` + `Sum`).
pub fn Sha256(key: String, msg: String) -> String {
    return sha256::HexOf(Sha256Bytes(key, msg));
}

/// HMAC-SHA-256 raw 32 bytes (Go's `mac.Sum(nil)`, list-adapted).
pub fn Sha256Bytes(key: String, msg: String) -> Dynamic {
    let kb = strbytes(key);
    if kb.len() > 64 {
        kb = sha256::SumBytes(key);
    }
    while kb.len() < 64 {
        kb.push(0);
    }
    let inner = [];
    let outer = [];
    let i = 0;
    while i < 64 {
        inner.push(kb[i] ^ 54);
        outer.push(kb[i] ^ 92);
        i = i + 1;
    }
    let ih = sha256::SumBytes(fromBytes(inner) + msg);
    return sha256::SumBytes(fromBytes(outer) + fromBytes(ih));
}

fn strbytes(s: String) -> Dynamic {
    let out = [];
    let i = 0;
    while i < s.len() {
        out.push(Text::code_at(s, i));
        i = i + 1;
    }
    return out;
}

fn fromBytes(b: Dynamic) -> String {
    let out = "";
    let i = 0;
    while i < b.len() {
        out = out + Text::from_code(b[i]);
        i = i + 1;
    }
    return out;
}
