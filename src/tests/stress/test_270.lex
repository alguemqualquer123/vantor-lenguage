module tests.test_270;

fn test_270() {
    let result = 89 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_270()
}
