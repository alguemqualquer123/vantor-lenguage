module tests.test_655;

fn test_655() {
    let result = 89 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_655()
}
