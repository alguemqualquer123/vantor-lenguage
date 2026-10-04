// e2e_113 - nullable type suffix
fn neg(x: i64?) -> i64 {
    if x > 0 {
        return 1;
    } else {
        return 0 - 1;
    }
    return 0;
}
pub fn main() -> void {
    let r = neg(3);
    return;
}
