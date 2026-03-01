module tests.test_970;

fn test_970() {
    let result = 14 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_970()
}
