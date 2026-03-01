module tests.test_127;

fn test_127() {
    let result = 32 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_127()
}
