// e2e_056 - match tuple pattern
fn first_of(p: (i64, i64)) -> i64 {
    match p {
        (a, b) => a,
        _ => 0
    }
    return 0;
}
pub fn main() -> void {
    let r = first_of(pair);
    return;
}
