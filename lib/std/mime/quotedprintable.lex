// Lexicon Standard Library — mime/quotedprintable.
// Go-parity quoted-printable codec (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 4) — the transfer encoding used by MIME body parts (RFC 2045 §6.7).
// Bytes cross as strings (code points 0-255 round-trip exactly, like the
// other byte codecs here). Import as `import std::mime::quotedprintable;`.

const qp_hex = "0123456789ABCDEF";

/// Result of a decode: the decoded `text` plus `ok` (Go's `(b, err)` pair).
pub struct QPDecoded {
    text: String,
    ok: bool,
}

/// Encodes `s` as quoted-printable, no line breaks (Go's
/// `quotedprintable.Encode` without the writer's 76-column wrapping).
pub fn Encode(s: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let b = Text::code_at(s, i);
        if b == 9 || b == 32 {
            out = out + Text::from_code(b);
        } else {
            if b >= 33 && b <= 126 && b != 61 {
                out = out + Text::from_code(b);
            } else {
                out = out + "=" + Text::slice(qp_hex, b / 16, b / 16 + 1);
                out = out + Text::slice(qp_hex, b % 16, b % 16 + 1);
            }
        }
        i = i + 1;
    }
    return out;
}

/// Decodes quoted-printable `s` (Go's `quotedprintable.Decode`). Accepts
/// `=XX` octets and `=` soft line breaks; reports `ok=false` on a truncated
/// or non-hex escape.
pub fn Decode(s: String) -> QPDecoded {
    let out = "";
    let i = 0;
    while i < s.len() {
        let b = Text::code_at(s, i);
        if b != 61 {
            out = out + Text::from_code(b);
            i = i + 1;
        } else {
            if i + 2 < s.len() && Text::code_at(s, i + 1) == 13 && Text::code_at(s, i + 2) == 10 {
                // Soft line break: "=\r\n".
                i = i + 3;
            } else if i + 2 < s.len() {
                let hi = qp_val(Text::code_at(s, i + 1));
                let lo = qp_val(Text::code_at(s, i + 2));
                if hi < 0 || lo < 0 {
                    return QPDecoded { text: out, ok: false };
                }
                out = out + Text::from_code(hi * 16 + lo);
                i = i + 3;
            } else {
                return QPDecoded { text: out, ok: false };
            }
        }
    }
    return QPDecoded { text: out, ok: true };
}

/// Byte value of a hex digit, or -1 (shared upper/lower case).
fn qp_val(c: i64) -> i64 {
    if c >= 48 && c <= 57 {
        return c - 48;
    }
    if c >= 65 && c <= 70 {
        return c - 55;
    }
    if c >= 97 && c <= 102 {
        return c - 87;
    }
    return -1;
}
