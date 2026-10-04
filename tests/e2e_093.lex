// e2e_093 - where clause with two constraints
fn pick<T, U>(x: T, y: U) where T: Show, U: Show -> T {
    return x;
}
pub fn main() -> void {
    let r = pick(1, 2);
    return;
}
