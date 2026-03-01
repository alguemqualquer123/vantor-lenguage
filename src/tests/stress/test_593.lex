module tests.test_593;

fn test_593() {
    let result = 77 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_593()
}
