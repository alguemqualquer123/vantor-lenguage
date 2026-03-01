module tests.test_460;

fn test_460() {
    let result = 35 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_460()
}
