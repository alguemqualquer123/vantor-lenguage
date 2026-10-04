// e2e_075 - recursion
fn fact(n: i64) -> i64 {
    if n < 2 {
        return 1;
    } else {
        return n * fact(n - 1);
    }
    return n;
}
pub fn main() -> void {
    let r = fact(5);
    return;
}
