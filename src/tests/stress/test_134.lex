module tests.test_134;

fn test_134() {
    let result = 33 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_134()
}
