module tests.test_660;

fn test_660() {
    let result = 3 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_660()
}
