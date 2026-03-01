module tests.test_100;

fn test_100() {
    let result = 24 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_100()
}
