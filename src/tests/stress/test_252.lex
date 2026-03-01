module tests.test_252;

fn test_252() {
    let result = 89 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_252()
}
