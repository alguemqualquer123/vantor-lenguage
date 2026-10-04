// Lexicon Standard Library — math.
// Go-parity float64 math (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
// Transcendentals delegate to the native `Math::*` builtins (exact,
// single-instruction, ~100x faster than series in the interpreter);
// rounding/clamp stay in Lex (already O(1)).

/// Archimedes' constant.
pub const Pi = 3.141592653589793;
/// e, the base of the natural logarithm.
pub const E = 2.718281828459045;
/// Phi, the golden ratio.
pub const Phi = 1.618033988749895;
/// Square root of 2.
pub const Sqrt2 = 1.4142135623730951;
/// Square root of e.
pub const SqrtE = 1.6487212707001282;
/// Square root of Pi.
pub const SqrtPi = 1.772453850905516;
/// Square root of Phi.
pub const SqrtPhi = 1.272019649514069;
/// Natural logarithm of 2.
pub const Ln2 = 0.6931471805599453;
/// Base-2 logarithm of e.
pub const Log2E = 1.4426950408889634;
/// Natural logarithm of 10.
pub const Ln10 = 2.302585092994046;
/// Base-10 logarithm of e.
pub const Log10E = 0.4342944819032518;

/// Returns the absolute value of `x` (Go's `math.Abs`).
pub fn Abs(x: f64) -> f64 {
    if x < 0.0 {
        return 0.0 - x;
    }
    return x;
}

/// Integer absolute value.
pub fn AbsInt(x: i64) -> i64 {
    if x < 0 {
        return 0 - x;
    }
    return x;
}

/// Returns -1, 0, or +1 according to the sign of `x` (Go's `math.Sign`;
/// Go spells it Signbit/Sign differences aside, this matches Sign(x)).
pub fn Sign(x: f64) -> f64 {
    if x < 0.0 {
        return -1.0;
    }
    if x > 0.0 {
        return 1.0;
    }
    return 0.0;
}

/// Returns the greatest integer value less than or equal to `x`.
pub fn Floor(x: f64) -> f64 {
    let t = x as i64;
    if x < 0.0 && (t as f64) != x {
        return (t - 1) as f64;
    }
    return t as f64;
}

/// Returns the smallest integer value greater than or equal to `x`.
pub fn Ceil(x: f64) -> f64 {
    let t = x as i64;
    if x > 0.0 && (t as f64) != x {
        return (t + 1) as f64;
    }
    return t as f64;
}

/// Returns the integer value of `x` truncated toward zero.
pub fn Trunc(x: f64) -> f64 {
    let t = x as i64;
    return t as f64;
}

/// Returns `x` rounded to the nearest integer, halves away from zero
/// (Go's `math.Round`).
pub fn Round(x: f64) -> f64 {
    if x >= 0.0 {
        return Floor(x + 0.5);
    }
    return Ceil(x - 0.5);
}

/// Returns the square root of `x` (native; NaN for negatives, like Go).
pub fn Sqrt(x: f64) -> f64 {
    return Math::sqrt(x);
}

/// Returns the cube root of `x` (native).
pub fn Cbrt(x: f64) -> f64 {
    return Math::cbrt(x);
}

/// Returns e raised to the power `x` (native).
pub fn Exp(x: f64) -> f64 {
    return Math::exp(x);
}

/// Returns the natural logarithm of `x` (native; NaN when `x <= 0`).
pub fn Ln(x: f64) -> f64 {
    return Math::ln(x);
}

/// Returns the logarithm of `x` in `base` (native).
pub fn Log(x: f64, base: f64) -> f64 {
    return Math::log(x, base);
}

/// Returns the base-10 logarithm of `x` (native).
pub fn Log10(x: f64) -> f64 {
    return Math::log10(x);
}

/// Returns the base-2 logarithm of `x` (native).
pub fn Log2(x: f64) -> f64 {
    return Math::log2(x);
}

/// Returns `x` raised to the power `y` (native).
pub fn Pow(x: f64, y: f64) -> f64 {
    return Math::pow(x, y);
}

/// Returns the floating-point remainder of x/y (Go's `math.Mod`, native).
pub fn Mod(x: f64, y: f64) -> f64 {
    return Math::fmod(x, y);
}

/// Returns the sine of `x` (radians, native — Go's `math.Sin`).
pub fn Sin(x: f64) -> f64 {
    return Math::sin(x);
}

/// Returns the cosine of `x` (radians, native — Go's `math.Cos`).
pub fn Cos(x: f64) -> f64 {
    return Math::cos(x);
}

/// Returns the tangent of `x` (radians, native — Go's `math.Tan`).
pub fn Tan(x: f64) -> f64 {
    return Math::tan(x);
}

/// Returns the arcsine of `x` (native — Go's `math.Asin`).
pub fn Asin(x: f64) -> f64 {
    return Math::asin(x);
}

/// Returns the arccosine of `x` (native — Go's `math.Acos`).
pub fn Acos(x: f64) -> f64 {
    return Math::acos(x);
}

/// Returns the arctangent of `x` (native — Go's `math.Atan`).
pub fn Atan(x: f64) -> f64 {
    return Math::atan(x);
}

/// Returns the arctangent of y/x using the signs to pick the quadrant
/// (native — Go's `math.Atan2`).
pub fn Atan2(y: f64, x: f64) -> f64 {
    return Math::atan2(y, x);
}

/// Returns the hypotenuse `sqrt(x*x + y*y)` without overflow
/// (native — Go's `math.Hypot`).
pub fn Hypot(x: f64, y: f64) -> f64 {
    return Math::hypot(x, y);
}

/// Returns the larger of `x` and `y`.
pub fn Max(x: f64, y: f64) -> f64 {
    if x >= y {
        return x;
    }
    return y;
}

/// Returns the smaller of `x` and `y`.
pub fn Min(x: f64, y: f64) -> f64 {
    if x <= y {
        return x;
    }
    return y;
}

/// Integer maximum.
pub fn MaxInt(x: i64, y: i64) -> i64 {
    if x >= y {
        return x;
    }
    return y;
}

/// Integer minimum.
pub fn MinInt(x: i64, y: i64) -> i64 {
    if x <= y {
        return x;
    }
    return y;
}

/// Returns `x` clamped to the inclusive range [lo, hi] (Go's `math.Clamp`,
/// added in Go 1.21).
pub fn Clamp(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo {
        return lo;
    }
    if x > hi {
        return hi;
    }
    return x;
}
