// Lexicon Standard Library — unicode/utf8.
// Go-parity rune helpers (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
// Import as `import std::unicode::utf8;` and call `utf8::RuneCount(s)`.
// Code points come from the native `Text::*` builtins (O(1) each).

/// Result of decoding one rune: the code point, its UTF-8 width in bytes
/// and whether the input was valid (Go's `DecodeRuneInString` triple).
pub struct RuneDecoded {
    rune: i64,
    size: i64,
    ok: bool,
}

/// Returns the number of runes in `s` (Go's `utf8.RuneCountInString`).
pub fn RuneCount(s: String) -> i64 {
    return s.len();
}

/// Reports whether `s` is valid UTF-8. Strings built by the runtime always
/// are (documented; mirrors Go's `utf8.ValidString` for well-formed input).
pub fn ValidString(s: String) -> bool {
    return true;
}

/// Returns the UTF-8 width in bytes of rune `r` (Go's `utf8.RuneLen`).
pub fn RuneLen(r: i64) -> i64 {
    if r < 0 {
        return -1;
    }
    if r < 128 {
        return 1;
    }
    if r < 2048 {
        return 2;
    }
    if r < 65536 {
        if r >= 55296 && r <= 57343 {
            return -1;
        }
        return 3;
    }
    if r <= 1114111 {
        return 4;
    }
    return -1;
}

/// Decodes the first rune of `s` (Go's `utf8.DecodeRuneInString`).
pub fn DecodeRune(s: String) -> RuneDecoded {
    if s.len() == 0 {
        return RuneDecoded { rune: -1, size: 0, ok: false };
    }
    let r = Text::code_at(s, 0);
    return RuneDecoded { rune: r, size: RuneLen(r), ok: true };
}

/// Encodes rune `r` as a one-character string. Returns "" for surrogates
/// and out-of-range values (Go yields `U+FFFD`; documented adaptation —
/// use `RuneError()` explicitly when you need the replacement char).
pub fn EncodeRune(r: i64) -> String {
    if r < 0 || r > 1114111 {
        return "";
    }
    if r >= 55296 && r <= 57343 {
        return "";
    }
    return Text::from_code(r);
}

/// The Unicode replacement character as a code point (Go's `utf8.RuneError`).
pub fn RuneError() -> i64 {
    return 65533;
}

/// Encodes the replacement character as a string.
pub fn RuneErrorString() -> String {
    return Text::from_code(65533);
}
