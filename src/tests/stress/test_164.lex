module tests.test_164;

fn test_164() {
    let result = 98 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_164()
}
