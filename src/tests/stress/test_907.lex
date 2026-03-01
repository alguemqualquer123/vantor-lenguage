module tests.test_907;

fn test_907() {
    let result = 4 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_907()
}
