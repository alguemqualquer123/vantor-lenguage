module tests.test_673;

fn test_673() {
    let result = 95 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_673()
}
