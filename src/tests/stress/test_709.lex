module tests.test_709;

fn test_709() {
    let result = 33 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_709()
}
