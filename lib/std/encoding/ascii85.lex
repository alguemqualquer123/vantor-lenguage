// Lexicon Standard Library — encoding/ascii85.
// Go-parity ascii85 (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). Adobe
// `z` shortcut + trailing `y` space shortcut included, like Go.
// Import as `import std::encoding::ascii85;`.

/// Result of a decode: the `text` plus `ok` (Go's `(data, err)` pair).
pub struct DecodeResult85 {
    text: String,
    ok: bool,
}

/// Encodes with `z`/`y` shortcuts (Go's `ascii85.Encode`).
pub fn Encode(s: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let n = s.len() - i;
        if n > 4 {
            n = 4;
        }
        let v = 0;
        let k = 0;
        while k < 4 {
            v = v * 256;
            if k < n {
                v = v + Text::code_at(s, i + k);
            }
            k = k + 1;
        }
        if n == 4 {
            if v == 0 {
                out = out + "z";
            } else {
                if v == 538976288 {
                    out = out + "y";
                } else {
                    out = out + five(v);
                }
            }
        } else {
            let full = five(v);
            out = out + Text::slice(full, 0, n + 1);
        }
        i = i + 4;
    }
    return out;
}

/// Decodes (Go's `ascii85.Decode`).
pub fn Decode(s: String) -> DecodeResult85 {
    let fail = DecodeResult85 { text: "", ok: false };
    let out = "";
    let i = 0;
    while i < s.len() {
        let c = Text::slice(s, i, i + 1);
        if c == " " || c == "\t" || c == "\r" || c == "\n" {
            i = i + 1;
            continue;
        }
        if c == "z" {
            out = out + Text::from_code(0) + Text::from_code(0) + Text::from_code(0) + Text::from_code(0);
            i = i + 1;
            continue;
        }
        if c == "y" {
            out = out + "    ";
            i = i + 1;
            continue;
        }
        let v = 0;
        let n = 0;
        while n < 5 && i < s.len() {
            let d = Text::slice(s, i, i + 1);
            if d == " " || d == "\t" || d == "\r" || d == "\n" {
                break;
            }
            let dv = Text::code_at(s, i) - 33;
            if dv < 0 || dv > 84 {
                return fail;
            }
            v = v * 85 + dv;
            n = n + 1;
            i = i + 1;
        }
        if n <= 1 {
            return fail;
        }
        // Short final groups pad with 'u' (value 84), like Go — then the
        // leading n-1 bytes are data.
        let m = n;
        while m < 5 {
            v = v * 85 + 84;
            m = m + 1;
        }
        let b0 = (v / 16777216) % 256;
        let b1 = (v / 65536) % 256;
        let b2 = (v / 256) % 256;
        let b3 = v % 256;
        if n >= 2 {
            out = out + Text::from_code(b0);
        }
        if n >= 3 {
            out = out + Text::from_code(b1);
        }
        if n >= 4 {
            out = out + Text::from_code(b2);
        }
        if n >= 5 {
            out = out + Text::from_code(b3);
        }
    }
    return DecodeResult85 { text: out, ok: true };
}

fn five(v: i64) -> String {
    let out = "";
    let d = 52200625;
    let k = 0;
    while k < 5 {
        out = out + Text::from_code(33 + (v / d) % 85);
        d = d / 85;
        k = k + 1;
    }
    return out;
}
