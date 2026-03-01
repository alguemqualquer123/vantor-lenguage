module tests.test_450;

fn test_450() {
    let result = 68 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_450()
}
