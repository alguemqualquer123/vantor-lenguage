module tests.test_952;

fn test_952() {
    let result = 31 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_952()
}
