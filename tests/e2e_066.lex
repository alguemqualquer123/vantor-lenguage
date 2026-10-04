// e2e_066 - defer multiple
fn a() -> void {
    return;
}
fn b() -> void {
    return;
}
fn work() -> void {
    defer a();
    defer b();
    let x = 1;
    return;
}
pub fn main() -> void {
    work();
    return;
}
