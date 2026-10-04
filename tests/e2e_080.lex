// e2e_080 - higher-order nested call
fn double(n: int) -> int {
    return n * 2;
}
fn twice(f: fn(int) -> int, x: int) -> int {
    return f(f(x));
}
pub fn main() -> void {
    let r = twice(double, 3);
    return;
}
