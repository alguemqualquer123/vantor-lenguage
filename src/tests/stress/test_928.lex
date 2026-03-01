module tests.test_928;

fn test_928() {
    let result = 87 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_928()
}
