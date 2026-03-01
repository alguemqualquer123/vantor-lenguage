module tests.test_54;

fn test_54() {
    let result = 65 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_54()
}
