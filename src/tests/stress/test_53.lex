module tests.test_53;

fn test_53() {
    let result = 58 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_53()
}
