// Lexicon Standard Library — encoding/base32.
// Go-parity base32 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). Pure Lex
// over the RFC 4648 alphabets. Import as `import std::encoding::base32;`.

const b32_std = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const b32_hex = "0123456789ABCDEFGHIJKLMNOPQRSTUV";

/// Result of a decode: the `text` plus `ok` (Go's `(data, err)` pair).
pub struct DecodeResult32 {
    text: String,
    ok: bool,
}

/// Encodes with the standard alphabet + padding (Go's
/// `base32.StdEncoding.EncodeToString`).
pub fn Encode(s: String) -> String {
    return b32_enc(s, b32_std);
}

/// Encodes with the extended-hex alphabet (Go's `HexEncoding`).
pub fn EncodeHex(s: String) -> String {
    return b32_enc(s, b32_hex);
}

/// Decodes standard base32 (Go's `StdEncoding.DecodeString`).
pub fn Decode(s: String) -> DecodeResult32 {
    return b32_dec(s, b32_std);
}

/// Decodes extended-hex base32.
pub fn DecodeHex(s: String) -> DecodeResult32 {
    return b32_dec(s, b32_hex);
}

fn b32_enc(s: String, alpha: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let bits = 0;
        let nbytes = 0;
        let k = 0;
        while k < 5 && i + k < s.len() {
            bits = bits * 256 + Text::code_at(s, i + k);
            nbytes = nbytes + 1;
            k = k + 1;
        }
        let total = nbytes * 8;
        while total > 0 {
            let shift = total - 5;
            if shift < 0 {
                shift = 0;
            }
            let v = 0;
            if total >= 5 {
                v = (bits / pow32(shift)) % 32;
            } else {
                v = (bits * pow32(5 - total)) % 32;
            }
            out = out + Text::slice(alpha, v, v + 1);
            total = total - 5;
        }
        let pad = 8 - ((nbytes * 8 + 4) / 5);
        if nbytes < 5 {
            let p = 0;
            while p < pad {
                out = out + "=";
                p = p + 1;
            }
        }
        i = i + 5;
    }
    return out;
}

fn pow32(e: i64) -> i64 {
    let p = 1;
    let i = 0;
    while i < e {
        p = p * 2;
        i = i + 1;
    }
    return p;
}

fn b32_val(c: String, alpha: String) -> i64 {
    if c == "=" {
        return -2;
    }
    return Text::index_of(alpha, c);
}

fn b32_dec(s: String, alpha: String) -> DecodeResult32 {
    let fail = DecodeResult32 { text: "", ok: false };
    if s.len() % 8 != 0 {
        return fail;
    }
    let out = "";
    let i = 0;
    while i < s.len() {
        let bits = 0;
        let nchars = 0;
        let k = 0;
        while k < 8 {
            let c = Text::slice(s, i + k, i + k + 1);
            if c == "=" {
                break;
            }
            let v = b32_val(c, alpha);
            if v < 0 {
                return fail;
            }
            bits = bits * 32 + v;
            nchars = nchars + 1;
            k = k + 1;
        }
        if nchars == 1 || nchars == 3 || nchars == 6 {
            return fail;
        }
        let nbytes = (nchars * 5) / 8;
        let b = 0;
        while b < nbytes {
            let shift = nchars * 5 - (b + 1) * 8;
            out = out + Text::from_code((bits / pow32(shift)) % 256);
            b = b + 1;
        }
        i = i + 8;
    }
    return DecodeResult32 { text: out, ok: true };
}
