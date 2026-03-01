module tests.test_1000;

fn test_1000() {
    let result = 82 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_1000()
}
