// e2e_077 - fn pointer param
fn inc(x: int) -> int {
    return x + 1;
}
fn apply(cb: fn(int) -> int, x: int) -> int {
    return cb(x);
}
pub fn main() -> void {
    let r = apply(inc, 1);
    return;
}
