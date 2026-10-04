// Lexicon Standard Library — encoding/base64.
// Go-parity base64 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). Pure Lex
// over the ASCII alphabet; strings cross as bytes (Latin-1 fast path,
// exact for ASCII). Import as `import std::encoding::base64;`.

const b64_std = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const b64_url = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Result of a decode: the `text` plus `ok` (Go's `(data, err)` pair).
pub struct B64Decoded {
    text: String,
    ok: bool,
}

/// Encodes `s` with the standard alphabet + padding (Go's
/// `base64.StdEncoding.EncodeToString`).
pub fn Encode(s: String) -> String {
    return b64_enc(s, b64_std, true);
}

/// Encodes with the URL-safe alphabet (Go's `URLEncoding`).
pub fn EncodeURL(s: String) -> String {
    return b64_enc(s, b64_url, true);
}

/// Encodes with the standard alphabet and no padding (Go's
/// `RawStdEncoding`).
pub fn EncodeRaw(s: String) -> String {
    return b64_enc(s, b64_std, false);
}

/// Decodes standard base64 (Go's `StdEncoding.DecodeString`).
pub fn Decode(s: String) -> B64Decoded {
    return b64_dec(s, b64_std);
}

/// Decodes URL-safe base64 (Go's `URLEncoding.DecodeString`).
pub fn DecodeURL(s: String) -> B64Decoded {
    return b64_dec(s, b64_url);
}

fn b64_enc(s: String, alpha: String, pad: bool) -> String {
    let out = "";
    let i = 0;
    let n = s.len();
    while i < n {
        let b0 = Text::code_at(s, i);
        let b1 = -1;
        let b2 = -1;
        if i + 1 < n {
            b1 = Text::code_at(s, i + 1);
        }
        if i + 2 < n {
            b2 = Text::code_at(s, i + 2);
        }
        let n0 = b0 / 4;
        let n1 = (b0 % 4) * 16;
        if b1 >= 0 {
            n1 = n1 + b1 / 16;
        }
        out = out + Text::slice(alpha, n0, n0 + 1);
        out = out + Text::slice(alpha, n1, n1 + 1);
        if b1 >= 0 {
            let n2 = (b1 % 16) * 4;
            if b2 >= 0 {
                n2 = n2 + b2 / 64;
            }
            out = out + Text::slice(alpha, n2, n2 + 1);
        } else {
            if pad {
                out = out + "=";
            }
        }
        if b2 >= 0 {
            out = out + Text::slice(alpha, b2 % 64, b2 % 64 + 1);
        } else {
            if pad {
                out = out + "=";
            }
        }
        i = i + 3;
    }
    return out;
}

fn b64_val(c: String, alpha: String) -> i64 {
    if c == "=" {
        return -2;
    }
    return Text::index_of(alpha, c);
}

fn b64_dec(s: String, alpha: String) -> B64Decoded {
    let fail = B64Decoded { text: "", ok: false };
    if s.len() % 4 != 0 {
        return fail;
    }
    let out = "";
    let i = 0;
    let n = s.len();
    while i < n {
        let c0 = Text::slice(s, i, i + 1);
        let c1 = Text::slice(s, i + 1, i + 2);
        let c2 = Text::slice(s, i + 2, i + 3);
        let c3 = Text::slice(s, i + 3, i + 4);
        let v0 = b64_val(c0, alpha);
        let v1 = b64_val(c1, alpha);
        if v0 < 0 || v1 < 0 {
            return fail;
        }
        out = out + Text::from_code(v0 * 4 + v1 / 16);
        if c2 != "=" {
            let v2 = b64_val(c2, alpha);
            if v2 < 0 {
                return fail;
            }
            out = out + Text::from_code((v1 % 16) * 16 + v2 / 4);
            if c3 != "=" {
                let v3 = b64_val(c3, alpha);
                if v3 < 0 {
                    return fail;
                }
                out = out + Text::from_code((v2 % 4) * 64 + v3);
            }
        }
        i = i + 4;
    }
    return B64Decoded { text: out, ok: true };
}
