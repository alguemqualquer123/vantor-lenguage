module tests.test_531;

fn test_531() {
    let result = 42 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_531()
}
