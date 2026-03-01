module tests.test_448;

fn test_448() {
    let result = 34 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_448()
}
