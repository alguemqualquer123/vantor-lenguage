// e2e_111 - Option type with Some and None
fn maybe(x: int) -> Option<int> {
    if x > 0 {
        return Some(x);
    } else {
        return None;
    }
    return None;
}
pub fn main() -> void {
    let r = maybe(1);
    return;
}
