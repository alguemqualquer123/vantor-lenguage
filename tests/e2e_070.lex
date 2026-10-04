// e2e_070 - defer plus panic plus recover
fn risky() -> void {
    defer recover();
    panic("x");
    return;
}
pub fn main() -> void {
    risky();
    return;
}
