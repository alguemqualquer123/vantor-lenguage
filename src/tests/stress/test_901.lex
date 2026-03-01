module tests.test_901;

fn test_901() {
    let result = 36 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_901()
}
