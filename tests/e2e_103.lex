// e2e_103 - class constructor and method
class Counter {
    count: int,
    constructor(n: int) {
        let c = n;
    }
    fn get(n: int) -> int {
        return n;
    }
}
pub fn main() -> void {
    let c = Counter { count: 0 };
    return;
}
