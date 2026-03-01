module tests.test_65;

fn test_65() {
    let result = 45 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_65()
}
