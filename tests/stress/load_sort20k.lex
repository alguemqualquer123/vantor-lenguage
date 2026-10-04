// load_sort20k — native sort over 20k reversed ints.
import std::sort;

pub fn main() -> void {
    let t0 = Time::now_unix_ms();
    let xs = [];
    let i = 20000;
    while i > 0 {
        xs.push(i);
        i = i - 1;
    }
    xs = sort::Ints(xs);
    assert(sort::IntsAreSorted(xs), "sorted");
    assert(xs[0] == 1 && xs[xs.len() - 1] == 20000, "ends");
    Console::log(Time::now_unix_ms() - t0);
    return;
}
