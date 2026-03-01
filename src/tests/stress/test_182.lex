module tests.test_182;

fn test_182() {
    let result = 98 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_182()
}
