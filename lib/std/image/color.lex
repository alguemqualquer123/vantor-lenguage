// Lexicon Standard Library — image/color.
// Go-parity color models (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 7).
// Channels ride 0-65535 (Go's 16-bit `RGBA()` values); 8-bit forms
// convert explicitly. Import as `import std::image::color;`.

/// 16-bit-premultiplied RGBA (Go's `color.RGBA` 8-bit form).
pub struct RGBA {
    r: i64,
    g: i64,
    b: i64,
    a: i64,
}

/// Non-premultiplied 8-bit form (Go's `color.NRGBA`).
pub struct NRGBA {
    r: i64,
    g: i64,
    b: i64,
    a: i64,
}

/// Grayscale 8-bit (Go's `color.Gray`).
pub struct Gray {
    y: i64,
}

/// 16-bit gray (Go's `color.Gray16`).
pub struct Gray16 {
    y: i64,
}

/// Expands 8-bit RGBA to 16-bit premultiplied quadruple
/// (Go's `Color.RGBA()`), as a list.
pub fn RGBAOf(c: RGBA) -> Dynamic {
    let r = c.r * 257;
    let g = c.g * 257;
    let b = c.b * 257;
    let a = c.a * 257;
    return [r, g, b, a];
}

/// Converts NRGBA to premultiplied RGBA (Go's `NRGBAModel.Convert`).
pub fn NRGBAtoRGBA(c: NRGBA) -> RGBA {
    if c.a == 255 {
        return RGBA { r: c.r, g: c.g, b: c.b, a: 255 };
    }
    if c.a == 0 {
        return RGBA { r: 0, g: 0, b: 0, a: 0 };
    }
    return RGBA { r: c.r * c.a / 255, g: c.g * c.a / 255, b: c.b * c.a / 255, a: c.a };
}

/// Converts RGBA to non-premultiplied form (Go's `RGBAModel` inverse).
pub fn RGBAtoNRGBA(c: RGBA) -> NRGBA {
    if c.a == 255 {
        return NRGBA { r: c.r, g: c.g, b: c.b, a: 255 };
    }
    if c.a == 0 {
        return NRGBA { r: 0, g: 0, b: 0, a: 0 };
    }
    return NRGBA { r: c.r * 255 / c.a, g: c.g * 255 / c.a, b: c.b * 255 / c.a, a: c.a };
}

/// Luminance grayscale (Go's `GrayModel.Convert`, Rec. 601 luma).
pub fn ToGray(c: RGBA) -> Gray {
    return Gray { y: (299 * c.r + 587 * c.g + 114 * c.b) / 1000 };
}

/// Parses `#rrggbb` / `#rgb` (CSS form accepted by image tools).
pub fn ParseHex(s: String) -> RGBA {
    let fail = RGBA { r: 0, g: 0, b: 0, a: 0 };
    let t = s;
    if t.len() > 0 && Text::slice(t, 0, 1) == "#" {
        t = Text::slice(t, 1, t.len());
    }
    if t.len() == 3 {
        let r = hex2(Text::slice(t, 0, 1) + Text::slice(t, 0, 1));
        let g = hex2(Text::slice(t, 1, 2) + Text::slice(t, 1, 2));
        let b = hex2(Text::slice(t, 2, 3) + Text::slice(t, 2, 3));
        if r < 0 || g < 0 || b < 0 {
            return fail;
        }
        return RGBA { r: r, g: g, b: b, a: 255 };
    }
    if t.len() == 6 {
        let r = hex2(Text::slice(t, 0, 2));
        let g = hex2(Text::slice(t, 2, 4));
        let b = hex2(Text::slice(t, 4, 6));
        if r < 0 || g < 0 || b < 0 {
            return fail;
        }
        return RGBA { r: r, g: g, b: b, a: 255 };
    }
    return fail;
}

/// Renders `#rrggbb` (inverse of `ParseHex` 6-digit form).
pub fn ToHex(c: RGBA) -> String {
    let digits = "0123456789abcdef";
    return "#" + hx(c.r) + hx(c.g) + hx(c.b);
}

fn hx(v: i64) -> String {
    let digits = "0123456789abcdef";
    return Text::slice(digits, v / 16, v / 16 + 1) + Text::slice(digits, v % 16, v % 16 + 1);
}

fn hex2(s: String) -> i64 {
    let digits = "0123456789abcdef";
    let hi = Text::index_of(digits, Text::slice(s, 0, 1).to_lower());
    let lo = Text::index_of(digits, Text::slice(s, 1, 2).to_lower());
    if hi < 0 || lo < 0 {
        return -1;
    }
    return hi * 16 + lo;
}
