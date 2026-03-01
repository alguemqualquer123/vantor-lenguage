module tests.test_259;

fn test_259() {
    let result = 16 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_259()
}
