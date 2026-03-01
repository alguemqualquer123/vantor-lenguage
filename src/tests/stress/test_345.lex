module tests.test_345;

fn test_345() {
    let result = 56 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_345()
}
