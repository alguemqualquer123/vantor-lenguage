module tests.test_711;

fn test_711() {
    let result = 45 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_711()
}
