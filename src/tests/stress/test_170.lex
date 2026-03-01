module tests.test_170;

fn test_170() {
    let result = 76 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_170()
}
