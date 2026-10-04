// e2e_048 - match wildcard and ident pattern
fn norm(x: int) -> int {
    match x {
        v => v,
        _ => 0
    }
    return x;
}
pub fn main() -> void {
    let r = norm(9);
    return;
}
