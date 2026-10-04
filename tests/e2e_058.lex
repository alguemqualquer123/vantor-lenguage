// e2e_058 - match guard plus wildcard combined
fn cls(x: int) -> int {
    match x {
        n if n < 0 => 0 - 1,
        0 => 0,
        _ => 1
    }
    return x;
}
pub fn main() -> void {
    let r = cls(0 - 5);
    return;
}
