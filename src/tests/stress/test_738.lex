module tests.test_738;

fn test_738() {
    let result = 92 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_738()
}
