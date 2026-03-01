module tests.test_95;

fn test_95() {
    let result = 56 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_95()
}
