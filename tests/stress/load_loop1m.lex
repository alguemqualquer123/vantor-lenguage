// load_loop1m — one million iterations with an exact accumulator.
pub fn main() -> void {
    let t0 = Time::now_unix_ms();
    let s = 0;
    let i = 0;
    while i < 1000000 {
        s = s + i;
        i = i + 1;
    }
    assert(s == 499999500000, "gauss sum");
    Console::log(Time::now_unix_ms() - t0);
    Console::log(s);
    return;
}
