// e2e_049 - match with guard
fn sign(x: int) -> int {
    match x {
        n if n > 0 => 1,
        _ => 0
    }
    return x;
}
pub fn main() -> void {
    let r = sign(5);
    return;
}
