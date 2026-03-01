module tests.test_966;

fn test_966() {
    let result = 79 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_966()
}
