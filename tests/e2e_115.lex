// e2e_115 - Option match integration
fn val(x: int) -> int {
    match x {
        0 => 0,
        _ => 1
    }
    return x;
}
pub fn main() -> void {
    let o = Some(5);
    let r = val(5);
    let n = None;
    return;
}
