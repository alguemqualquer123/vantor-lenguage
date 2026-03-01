module tests.test_534;

fn test_534() {
    let result = 85 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_534()
}
