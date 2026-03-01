module tests.test_113;

fn test_113() {
    let result = 40 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_113()
}
