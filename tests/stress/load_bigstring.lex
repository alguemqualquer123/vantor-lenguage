// load_bigstring — 200KB built by append (exercises the O(n) fast path).
pub fn main() -> void {
    let t0 = Time::now_unix_ms();
    let o = "";
    let k = 0;
    while k < 200000 {
        o = o + "x";
        k = k + 1;
    }
    assert(o.len() == 200000, "length");
    Console::log(Time::now_unix_ms() - t0);
    return;
}
