// e2e_065 - defer simple
fn cleanup() -> void {
    let x = 1;
    return;
}
fn work() -> void {
    defer cleanup();
    let x = 1;
    return;
}
pub fn main() -> void {
    work();
    return;
}
