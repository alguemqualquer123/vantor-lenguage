module tests.test_786;

fn test_786() {
    let result = 82 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_786()
}
