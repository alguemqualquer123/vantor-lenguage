module tests.test_701;

fn test_701() {
    let result = 36 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_701()
}
