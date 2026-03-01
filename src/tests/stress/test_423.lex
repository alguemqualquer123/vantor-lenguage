module tests.test_423;

fn test_423() {
    let result = 44 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_423()
}
