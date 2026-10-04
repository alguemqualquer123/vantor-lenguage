// e2e_052 - match bool exhaustive
fn bit(b: bool) -> i64 {
    match b {
        true => 1,
        false => 0
    }
    return 0;
}
pub fn main() -> void {
    let r = bit(true);
    return;
}
