module tests.test_526;

fn test_526() {
    let result = 90 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_526()
}
