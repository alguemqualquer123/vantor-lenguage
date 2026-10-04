// e2e_123 - Go-parity stdlib: math natives + module consts
module e2e_123;
import std::math;

pub fn main() -> void {
    assert(math::Pi > 3.14 && math::Pi < 3.15, "Pi const");
    assert(math::E > 2.71 && math::E < 2.72, "E const");
    assert(math::Sqrt(16.0) == 4.0, "Sqrt exact");
    assert(math::Pow(2.0, 10.0) == 1024.0, "Pow exact");
    assert(math::Sin(0.0) == 0.0, "Sin");
    assert(math::Cos(0.0) == 1.0, "Cos");
    assert(math::Exp(0.0) == 1.0, "Exp");
    assert(math::Ln(1.0) == 0.0, "Ln");
    assert(math::Log10(100.0) == 2.0, "Log10");
    assert(math::Log2(8.0) == 3.0, "Log2");
    assert(math::Hypot(3.0, 4.0) == 5.0, "Hypot");
    assert(math::Floor(2.7) == 2.0, "Floor");
    assert(math::Ceil(2.2) == 3.0, "Ceil");
    assert(math::Round(2.5) == 3.0, "Round");
    assert(math::Abs(0.0 - 5.0) == 5.0, "Abs");
    assert(math::Max(3.0, 7.0) == 7.0, "Max");
    assert(math::Min(3.0, 7.0) == 3.0, "Min");
    assert(math::Clamp(9.0, 0.0, 5.0) == 5.0, "Clamp");
    assert(Math::sqrt(9.0) == 3.0, "native Math");
    return;
}
