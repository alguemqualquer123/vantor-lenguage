module tests.test_29;

fn test_29() {
    let result = 16 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_29()
}
