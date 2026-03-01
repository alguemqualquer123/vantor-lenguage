module tests.test_99;

fn test_99() {
    let result = 31 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_99()
}
