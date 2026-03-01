module tests.test_425;

fn test_425() {
    let result = 8 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_425()
}
