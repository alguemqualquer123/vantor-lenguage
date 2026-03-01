module tests.test_21;

fn test_21() {
    let result = 23 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_21()
}
