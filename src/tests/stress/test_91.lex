module tests.test_91;

fn test_91() {
    let result = 49 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_91()
}
