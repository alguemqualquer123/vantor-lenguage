// atomic_demo.lex — shared counter with std::sync::atomic.
// Run: lex run examples/atomic_demo.lex
import std::sync::atomic;

pub fn main() -> void {
    let hits = atomic::NewInt64(0);
    let i = 0;
    while i < 1000 {
        atomic::Add(hits, 1);
        i = i + 1;
    }
    Console::log(atomic::Load(hits));
}
