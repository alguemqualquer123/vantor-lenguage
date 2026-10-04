// e2e_109 - type alias chain
type A = int;
type B = A;
pub fn main() -> void {
    let x: B = 1;
    return;
}
