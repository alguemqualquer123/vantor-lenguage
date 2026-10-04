// e2e_057 - match enum variant pattern
enum Color {
    Red,
    Green,
    Blue
}
fn is_red(c: Color) -> bool {
    match c {
        Red() => true,
        _ => false
    }
    return true;
}
pub fn main() -> void {
    let r = is_red(Red);
    return;
}
