module tests.test_585;

fn test_585() {
    let result = 47 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_585()
}
