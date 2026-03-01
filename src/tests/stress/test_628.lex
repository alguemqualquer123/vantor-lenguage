module tests.test_628;

fn test_628() {
    let result = 76 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_628()
}
