// e2e_079 - tuple typed destructuring
fn pair() -> (i64, i64) {
    return (1, 2);
}
pub fn main() -> void {
    let (a, b): (i64, i64) = pair();
    return;
}
