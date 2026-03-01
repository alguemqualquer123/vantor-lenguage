module tests.test_17;

fn test_17() {
    let result = 50 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_17()
}
