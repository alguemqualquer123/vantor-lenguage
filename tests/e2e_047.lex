// e2e_047 - match int arms with wildcard
fn pick(x: int) -> int {
    match x {
        1 => 10,
        2 => 20,
        _ => 0
    }
    return x;
}
pub fn main() -> void {
    let r = pick(1);
    return;
}
