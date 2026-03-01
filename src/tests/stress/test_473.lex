module tests.test_473;

fn test_473() {
    let result = 21 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_473()
}
