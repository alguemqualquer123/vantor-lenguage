// e2e_142 - std math/cmplx: complex128 parity with Go's cmplx package
module e2e_142;

import std::math::cmplx;

fn close(a: f64, b: f64) -> bool {
    let d = a - b;
    if d < 0.0 {
        d = 0.0 - d;
    }
    return d < 0.0000000001;
}

pub fn main() -> void {
    let z = cmplx::Make(3.0, 4.0);
    assert(close(cmplx::Abs(z), 5.0), "abs");
    assert(close(cmplx::Phase(z), 0.9272952180016122), "phase");
    assert(cmplx::Real(z) == 3.0 && cmplx::Imag(z) == 4.0, "real/imag");

    let r = cmplx::Sqrt(z);
    assert(close(r.re, 2.0) && close(r.im, 1.0), "sqrt 3+4i");
    let rn = cmplx::Sqrt(cmplx::Make(-4.0, 0.0));
    assert(close(rn.re, 0.0) && close(rn.im, 2.0), "sqrt -4");
    let rz = cmplx::Sqrt(cmplx::Make(-1.0, -1.0));
    assert(close(rz.re, 0.45508986056222733) && close(rz.im, -1.09868411346781), "sqrt -1-i");

    let w = cmplx::Make(1.0, 2.0);
    let d = cmplx::Div(z, w);
    assert(close(d.re, 2.2) && close(d.im, -0.4), "div");
    let m = cmplx::Mul(z, w);
    assert(close(m.re, -5.0) && close(m.im, 10.0), "mul");
    let a = cmplx::Add(z, w);
    assert(close(a.re, 4.0) && close(a.im, 6.0), "add");
    let s = cmplx::Sub(z, w);
    assert(close(s.re, 2.0) && close(s.im, 2.0), "sub");
    let g = cmplx::Neg(z);
    assert(close(g.re, -3.0) && close(g.im, -4.0), "neg");
    let k = cmplx::Scale(z, 2.0);
    assert(close(k.re, 6.0) && close(k.im, 8.0), "scale");
    let c = cmplx::Conj(z);
    assert(close(c.re, 3.0) && close(c.im, -4.0), "conj");

    let e = cmplx::Exp(cmplx::Make(0.0, 1.0));
    assert(close(e.re, 0.5403023058681398) && close(e.im, 0.8414709848078965), "exp i");
    let l = cmplx::Log(cmplx::Make(1.0, 1.0));
    assert(close(l.re, 0.3465735902799727) && close(l.im, 0.7853981633974483), "log 1+i");
    let l10 = cmplx::Log10(z);
    assert(close(l10.re, 0.6989700043360187) && close(l10.im, 0.4027191962733731), "log10");
    let l2 = cmplx::Log2(z);
    assert(close(l2.re, 2.321928094887362) && close(l2.im, 1.3378042124509761), "log2");
    let p = cmplx::PowInt(w, 5);
    assert(close(p.re, 41.0) && close(p.im, -38.0), "powint");
    let pn = cmplx::PowInt(w, -1);
    assert(close(pn.re, 0.2) && close(pn.im, -0.4), "powint neg");
    let pp = cmplx::Pow(z, cmplx::Make(2.0, 0.0));
    assert(close(pp.re, -7.0) && close(pp.im, 24.0), "pow");

    let sn = cmplx::Sin(cmplx::Make(1.0, 1.0));
    assert(close(sn.re, 1.2984575814159773) && close(sn.im, 0.6349639147847361), "sin");
    let cs = cmplx::Cos(cmplx::Make(1.0, 1.0));
    assert(close(cs.re, 0.8337300251311491) && close(cs.im, -0.9888977057628651), "cos");
    let tn = cmplx::Tan(cmplx::Make(0.5, -0.25));
    assert(close(tn.re, 0.504500702698564) && close(tn.im, -0.31242069250258875), "tan");
    let zero = cmplx::Sqrt(cmplx::Make(0.0, 0.0));
    assert(close(zero.re, 0.0) && close(zero.im, 0.0), "sqrt zero");
}
