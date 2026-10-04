// Lexicon Standard Library — unicode/utf16.
// Go-parity UTF-16 surrogate handling
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1). Units ride `Dynamic`
// lists of i64. Import as `import std::unicode::utf16;`.

/// Reports whether `r` is a surrogate half (Go's `utf16.IsSurrogate`).
pub fn IsSurrogate(r: i64) -> bool {
    return r >= 55296 && r <= 57343;
}

/// Decodes a surrogate pair (Go's `utf16.DecodeRune`).
pub fn DecodeRune(hi: i64, lo: i64) -> i64 {
    if hi < 55296 || hi > 56319 || lo < 56320 || lo > 57343 {
        return 65533;
    }
    return (hi - 55296) * 1024 + (lo - 56320) + 65536;
}

/// Encodes a rune into 1-2 units (Go's `utf16.EncodeRune`).
pub fn EncodeRune(r: i64) -> Dynamic {
    if r < 0 || r > 1114111 || IsSurrogate(r) {
        return [65533];
    }
    if r < 65536 {
        return [r];
    }
    let v = r - 65536;
    return [55296 + v / 1024, 56320 + v % 1024];
}

/// Encodes a string into units (Go's `utf16.Encode`).
pub fn Encode(s: String) -> Dynamic {
    let out = [];
    let i = 0;
    while i < s.len() {
        let parts = EncodeRune(Text::code_at(s, i));
        let j = 0;
        while j < parts.len() {
            out.push(parts[j]);
            j = j + 1;
        }
        i = i + 1;
    }
    return out;
}

/// Decodes units into a string, replacing broken pairs (Go's
/// `utf16.Decode` with `RuneError` substitution).
pub fn Decode(units: Dynamic) -> String {
    let out = "";
    let i = 0;
    while i < units.len() {
        let r = units[i];
        if r >= 55296 && r <= 56319 && i + 1 < units.len() {
            let lo = units[i + 1];
            if lo >= 56320 && lo <= 57343 {
                out = out + Text::from_code(DecodeRune(r, lo));
                i = i + 2;
                continue;
            }
        }
        if IsSurrogate(r) {
            out = out + Text::from_code(65533);
        } else {
            out = out + Text::from_code(r);
        }
        i = i + 1;
    }
    return out;
}
