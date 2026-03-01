module tests.test_223;

fn test_223() {
    let result = 33 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_223()
}
