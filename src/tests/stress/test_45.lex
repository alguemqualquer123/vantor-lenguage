module tests.test_45;

fn test_45() {
    let result = 72 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_45()
}
