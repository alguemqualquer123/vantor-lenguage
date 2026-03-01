module tests.test_489;

fn test_489() {
    let result = 57 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_489()
}
