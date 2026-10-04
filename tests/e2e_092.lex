// e2e_092 - generic enum and Some value
enum Opt<T> {
    None,
    Some
}
pub fn main() -> void {
    let o = Some(1);
    let x = 0;
    match x {
        0 => 0,
        _ => 1
    }
    return;
}
