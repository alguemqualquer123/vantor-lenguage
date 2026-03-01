module tests.test_22;

fn test_22() {
    let result = 23 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_22()
}
