// e2e_094 - generic struct with two params
struct Pair<A, B> {
    first: A,
    second: B
}
pub fn main() -> void {
    let p = Pair { first: 1, second: "a" };
    return;
}
