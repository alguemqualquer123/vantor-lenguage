module tests.test_218;

fn test_218() {
    let result = 48 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_218()
}
