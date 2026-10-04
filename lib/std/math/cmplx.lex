// Lexicon Standard Library — math/cmplx.
// Go-parity complex128 math (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 6).
// Values are Cartesian `Complex { re, im }` (Go's `complex128`); the
// transcendental functions use the standard polar / C99 Annex G
// decompositions over the native `Math::*` primitives.
// Import as `import std::math::cmplx;`.

/// A complex number (Go's `complex128`).
pub struct Complex {
    re: f64,
    im: f64,
}

/// Builds a value from parts (Go's `complex`).
pub fn Make(re: f64, im: f64) -> Complex {
    return Complex { re: re, im: im };
}

pub fn Real(z: Complex) -> f64 {
    return z.re;
}

pub fn Imag(z: Complex) -> f64 {
    return z.im;
}

/// Modulus |z| (Go's `cmplx.Abs`); `hypot` keeps large parts from
/// overflowing.
pub fn Abs(z: Complex) -> f64 {
    return Math::hypot(z.re, z.im);
}

/// Phase angle in (-pi, pi] (Go's `cmplx.Phase`).
pub fn Phase(z: Complex) -> f64 {
    return Math::atan2(z.im, z.re);
}

/// Component-wise sum (Go's `z1 + z2`).
pub fn Add(a: Complex, b: Complex) -> Complex {
    return Complex { re: a.re + b.re, im: a.im + b.im };
}

/// Component-wise difference (Go's `z1 - z2`).
pub fn Sub(a: Complex, b: Complex) -> Complex {
    return Complex { re: a.re - b.re, im: a.im - b.im };
}

/// Negation (Go's `-z`).
pub fn Neg(z: Complex) -> Complex {
    return Complex { re: 0.0 - z.re, im: 0.0 - z.im };
}

/// Scale by a real factor (Go's `z * scalar`).
pub fn Scale(z: Complex, k: f64) -> Complex {
    return Complex { re: z.re * k, im: z.im * k };
}

/// Complex product (Go's `z1 * z2`).
pub fn Mul(a: Complex, b: Complex) -> Complex {
    return Complex { re: a.re * b.re - a.im * b.im, im: a.re * b.im + a.im * b.re };
}

/// Complex quotient (Go's `z1 / z2`) using Smith's ratio formula, which
/// avoids the overflow of the naive `(a·conj(b)) / |b|²`.
pub fn Div(a: Complex, b: Complex) -> Complex {
    if b.re == 0.0 && b.im == 0.0 {
        panic("cmplx.Div by zero");
    }
    if Math::abs(b.re) >= Math::abs(b.im) {
        let r = b.im / b.re;
        let d = b.re + b.im * r;
        return Complex { re: (a.re + a.im * r) / d, im: (a.im - a.re * r) / d };
    }
    let r = b.re / b.im;
    let d = b.im + b.re * r;
    return Complex { re: (a.re * r + a.im) / d, im: (a.im * r - a.re) / d };
}

/// Conjugate (Go's `cmplx.Conj`).
pub fn Conj(z: Complex) -> Complex {
    return Complex { re: z.re, im: 0.0 - z.im };
}

/// Principal square root (Go's `cmplx.Sqrt`; C99 Annex G branch).
pub fn Sqrt(z: Complex) -> Complex {
    let re = z.re;
    let im = z.im;
    if re == 0.0 && im == 0.0 {
        return Complex { re: 0.0, im: im };
    }
    let t = Math::sqrt((Abs(z) + Math::abs(re)) / 2.0);
    if re >= 0.0 {
        return Complex { re: t, im: im / (2.0 * t) };
    }
    let wi = t;
    if im < 0.0 {
        wi = 0.0 - t;
    }
    return Complex { re: Math::abs(im) / (2.0 * t), im: wi };
}

/// e^z (Go's `cmplx.Exp`).
pub fn Exp(z: Complex) -> Complex {
    let m = Math::exp(z.re);
    return Complex { re: m * Math::cos(z.im), im: m * Math::sin(z.im) };
}

/// Principal natural logarithm (Go's `cmplx.Log`).
pub fn Log(z: Complex) -> Complex {
    return Complex { re: Math::ln(Abs(z)), im: Phase(z) };
}

/// Base-2 logarithm (Go's `cmplx.Log2`).
pub fn Log2(z: Complex) -> Complex {
    return Div(Log(z), Complex { re: Math::ln(2.0), im: 0.0 });
}

/// Base-10 logarithm (Go's `cmplx.Log10`).
pub fn Log10(z: Complex) -> Complex {
    return Div(Log(z), Complex { re: Math::ln(10.0), im: 0.0 });
}

/// z**y = e^(y·log z) (Go's `cmplx.Pow`).
pub fn Pow(z: Complex, y: Complex) -> Complex {
    return Exp(Mul(y, Log(z)));
}

/// z**n for an integer exponent by repeated squaring — exact for whole
/// powers, unlike the `Pow`/`Log` round trip.
pub fn PowInt(z: Complex, n: i64) -> Complex {
    if n == 0 {
        return Complex { re: 1.0, im: 0.0 };
    }
    let e = n;
    if e < 0 {
        e = 0 - e;
    }
    let acc = Complex { re: 1.0, im: 0.0 };
    let p = z;
    let k = e;
    while k > 0 {
        if k % 2 == 1 {
            acc = Mul(acc, p);
        }
        p = Mul(p, p);
        k = k / 2;
    }
    if n < 0 {
        return Div(Complex { re: 1.0, im: 0.0 }, acc);
    }
    return acc;
}

/// sin z (Go's `cmplx.Sin`).
pub fn Sin(z: Complex) -> Complex {
    return Complex { re: Math::sin(z.re) * cosh(z.im), im: Math::cos(z.re) * sinh(z.im) };
}

/// cos z (Go's `cmplx.Cos`).
pub fn Cos(z: Complex) -> Complex {
    return Complex { re: Math::cos(z.re) * cosh(z.im), im: 0.0 - Math::sin(z.re) * sinh(z.im) };
}

/// tan z (Go's `cmplx.Tan`).
pub fn Tan(z: Complex) -> Complex {
    return Div(Sin(z), Cos(z));
}

fn sinh(x: f64) -> f64 {
    let e = Math::exp(x);
    return (e - 1.0 / e) / 2.0;
}

fn cosh(x: f64) -> f64 {
    let e = Math::exp(x);
    return (e + 1.0 / e) / 2.0;
}
