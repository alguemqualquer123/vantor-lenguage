module tests.test_896;

fn test_896() {
    let result = 79 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_896()
}
