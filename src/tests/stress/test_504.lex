module tests.test_504;

fn test_504() {
    let result = 61 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_504()
}
