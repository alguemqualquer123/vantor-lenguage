module tests.test_703;

fn test_703() {
    let result = 39 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_703()
}
