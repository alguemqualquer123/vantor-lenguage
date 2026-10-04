// e2e_076 - variadic fn
fn sum(first: int, rest: int...) -> int {
    return first;
}
pub fn main() -> void {
    let r = sum(1, 2);
    return;
}
