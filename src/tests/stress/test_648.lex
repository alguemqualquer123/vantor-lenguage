module tests.test_648;

fn test_648() {
    let result = 42 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_648()
}
