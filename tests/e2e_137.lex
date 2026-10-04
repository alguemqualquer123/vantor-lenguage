// e2e_137 - Go-parity stdlib: unsafe, runtime, testing
module e2e_137;
import std::unsafe;
import std::runtime;
import std::testing;
import std::strings;

pub fn main() -> void {
    // unsafe word model
    assert(unsafe::Sizeof(1) == 8, "Sizeof int");
    assert(unsafe::Sizeof("ab") == 2, "Sizeof string");
    assert(unsafe::Alignof(1) == 8, "Alignof");
    assert(!unsafe::IsNil(0), "IsNil int");

    // runtime facts (real values)
    assert(runtime::GOOS() != "", "GOOS");
    assert(runtime::GOARCH() != "", "GOARCH");
    assert(runtime::NumCPU() >= 1, "NumCPU");
    assert(runtime::NumGoroutine() == 1, "NumGoroutine");
    assert(runtime::GOMAXPROCS(4) == 4, "GOMAXPROCS");
    assert(strings::HasPrefix(runtime::Version(), "lex0."), "Version tracks toolchain");

    // testing helpers
    let t = testing::New("demo");
    t = testing::AssertEq(t, 1, 1, "eq");
    assert(!testing::Failed(t), "not failed");
    t = testing::Assert(t, false, "boom");
    assert(testing::Failed(t), "failed");
    let b = testing::Benchmark(10, | | 0);
    assert(b.n == 10, "bench n");
    return;
}
