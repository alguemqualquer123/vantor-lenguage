module tests.test_328;

fn test_328() {
    let result = 81 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_328()
}
