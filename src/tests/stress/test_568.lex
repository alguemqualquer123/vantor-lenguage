module tests.test_568;

fn test_568() {
    let result = 23 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_568()
}
