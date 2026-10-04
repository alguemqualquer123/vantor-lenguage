// e2e_086 - pipe closure as call argument
fn apply(cb: fn(int) -> int, x: int) -> int {
    return cb(x);
}
pub fn main() -> void {
    let r = apply(0 |v| v + 1, 5);
    return;
}
