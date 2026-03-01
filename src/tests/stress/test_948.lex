module tests.test_948;

fn test_948() {
    let result = 87 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_948()
}
