module tests.test_537;

fn test_537() {
    let result = 44 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_537()
}
