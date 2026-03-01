module tests.test_385;

fn test_385() {
    let result = 39 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_385()
}
