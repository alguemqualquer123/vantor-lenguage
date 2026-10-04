// e2e_055 - match ident binding pattern
fn ident_pat(x: int) -> int {
    match x {
        val => val,
        _ => 0
    }
    return x;
}
pub fn main() -> void {
    let r = ident_pat(4);
    return;
}
