// Lexicon Standard Library — encoding/hex.
// Go-parity hexadecimal codec (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4).
// Import as `import std::encoding::hex;`.

const hex_digits = "0123456789abcdef";

/// Result of a decode: the `text` plus `ok` (Go's `(data, err)` pair).
pub struct HexDecoded {
    text: String,
    ok: bool,
}

/// Encodes `s` as lowercase hex (Go's `hex.EncodeToString`).
pub fn Encode(s: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let b = Text::code_at(s, i);
        out = out + Text::slice(hex_digits, b / 16, b / 16 + 1);
        out = out + Text::slice(hex_digits, b % 16, b % 16 + 1);
        i = i + 1;
    }
    return out;
}

/// Decodes lowercase (or uppercase) hex (Go's `hex.DecodeString`).
pub fn Decode(s: String) -> HexDecoded {
    let fail = HexDecoded { text: "", ok: false };
    if s.len() % 2 != 0 {
        return fail;
    }
    let out = "";
    let i = 0;
    while i < s.len() {
        let hi = hex_val(Text::slice(s, i, i + 1));
        let lo = hex_val(Text::slice(s, i + 1, i + 2));
        if hi < 0 || lo < 0 {
            return fail;
        }
        out = out + Text::from_code(hi * 16 + lo);
        i = i + 2;
    }
    return HexDecoded { text: out, ok: true };
}

fn hex_val(c: String) -> i64 {
    let v = Text::index_of(hex_digits, c.to_lower());
    return v;
}
