// e2e_089 - generic struct and literal
struct Box<T> {
    value: T
}
pub fn main() -> void {
    let b = Box { value: 42 };
    return;
}
