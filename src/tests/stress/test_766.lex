module tests.test_766;

fn test_766() {
    let result = 24 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_766()
}
