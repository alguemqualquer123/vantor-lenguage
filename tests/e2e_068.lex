// e2e_068 - panic call
fn boom() -> void {
    panic("boom");
    return;
}
pub fn main() -> void {
    boom();
    return;
}
