// e2e_088 - generic fn with two params
fn first<A, B>(a: A, b: B) -> A {
    return a;
}
pub fn main() -> void {
    let r = first(1, "s");
    return;
}
