module tests.test_10;

fn test_10() {
    let result = 10 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_10()
}
