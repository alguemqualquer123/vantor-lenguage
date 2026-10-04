// e2e_067 - defer in main
fn done() -> void {
    return;
}
pub fn main() -> void {
    defer done();
    let x = 1;
    return;
}
