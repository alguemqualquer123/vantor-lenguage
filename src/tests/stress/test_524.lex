module tests.test_524;

fn test_524() {
    let result = 30 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_524()
}
