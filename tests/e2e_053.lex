// e2e_053 - match with return arms
fn early(x: int) -> int {
    match x {
        1 => return 10,
        _ => return 0
    }
}
pub fn main() -> void {
    let r = early(1);
    return;
}
