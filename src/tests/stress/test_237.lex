module tests.test_237;

fn test_237() {
    let result = 38 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_237()
}
