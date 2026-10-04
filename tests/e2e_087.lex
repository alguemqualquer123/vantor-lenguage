// e2e_087 - generic fn
fn id<T>(x: T) -> T {
    return x;
}
pub fn main() -> void {
    let r = id(42);
    return;
}
