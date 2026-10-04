// e2e_050 - match with multiple guards
fn band(x: int) -> int {
    match x {
        n if n > 10 => 3,
        n if n > 0 => 2,
        _ => 1
    }
    return x;
}
pub fn main() -> void {
    let r = band(7);
    return;
}
