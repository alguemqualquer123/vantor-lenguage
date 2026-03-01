module tests.test_831;

fn test_831() {
    let result = 27 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_831()
}
