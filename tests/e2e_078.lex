// e2e_078 - tuple return and destructuring
fn pair() -> (i64, i64) {
    return (1, 2);
}
pub fn main() -> void {
    let (a, b) = pair();
    return;
}
