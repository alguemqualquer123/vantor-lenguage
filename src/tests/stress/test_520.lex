module tests.test_520;

fn test_520() {
    let result = 85 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_520()
}
