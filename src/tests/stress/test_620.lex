module tests.test_620;

fn test_620() {
    let result = 23 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_620()
}
