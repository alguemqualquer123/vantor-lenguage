module tests.test_400;

fn test_400() {
    let result = 23 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_400()
}
