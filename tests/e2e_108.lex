// e2e_108 - generic type alias
type Wrap<T> = T;
pub fn main() -> void {
    let x: Wrap<int> = 1;
    return;
}
