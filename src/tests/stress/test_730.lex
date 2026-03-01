module tests.test_730;

fn test_730() {
    let result = 80 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_730()
}
