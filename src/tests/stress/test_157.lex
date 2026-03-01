module tests.test_157;

fn test_157() {
    let result = 12 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_157()
}
