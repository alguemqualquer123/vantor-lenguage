module tests.test_197;

fn test_197() {
    let result = 69 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_197()
}
