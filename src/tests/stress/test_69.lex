module tests.test_69;

fn test_69() {
    let result = 58 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_69()
}
