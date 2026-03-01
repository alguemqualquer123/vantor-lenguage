module tests.test_74;

fn test_74() {
    let result = 48 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_74()
}
