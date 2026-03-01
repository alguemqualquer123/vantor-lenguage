module tests.test_401;

fn test_401() {
    let result = 63 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_401()
}
