// e2e_098 - nested struct literal
struct Inner {
    v: int
}
struct Outer {
    inner: Inner,
    n: int
}
pub fn main() -> void {
    let o = Outer { inner: Inner { v: 1 }, n: 2 };
    return;
}
