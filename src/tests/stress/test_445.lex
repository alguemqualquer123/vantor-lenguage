module tests.test_445;

fn test_445() {
    let result = 36 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_445()
}
