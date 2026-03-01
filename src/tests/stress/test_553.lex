module tests.test_553;

fn test_553() {
    let result = 38 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_553()
}
