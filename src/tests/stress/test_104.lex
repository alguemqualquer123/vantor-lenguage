module tests.test_104;

fn test_104() {
    let result = 39 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_104()
}
