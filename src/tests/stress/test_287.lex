module tests.test_287;

fn test_287() {
    let result = 87 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_287()
}
