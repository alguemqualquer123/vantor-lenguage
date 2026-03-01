module tests.test_694;

fn test_694() {
    let result = 10 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_694()
}
