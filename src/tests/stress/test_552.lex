module tests.test_552;

fn test_552() {
    let result = 85 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_552()
}
