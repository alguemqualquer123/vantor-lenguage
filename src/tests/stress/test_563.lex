module tests.test_563;

fn test_563() {
    let result = 2 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_563()
}
