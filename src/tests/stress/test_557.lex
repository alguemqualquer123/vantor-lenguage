module tests.test_557;

fn test_557() {
    let result = 97 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_557()
}
