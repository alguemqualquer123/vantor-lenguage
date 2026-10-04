// load_recursion_over_fail — MUST exit non-zero with a clean
// "call stack overflow" diagnostic (never a process abort).
pub fn down(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    return down(n - 1) + 1;
}

pub fn main() -> void {
    Console::log(down(20000));
    return;
}
