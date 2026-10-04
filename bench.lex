pub fn main() -> void {
    let mut i = 0;
    let mut acc = 0.0;
    let t0 = Time::now_unix_ms();
    while i < 100000 {
        acc = acc + i * 1.5 + 2.0;
        i = i + 1;
    }
    let t1 = Time::now_unix_ms();
    Console::log("ms=" + (t1 - t0).to_string() + " acc=" + acc.to_string());
}
