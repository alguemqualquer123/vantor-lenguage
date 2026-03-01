module tests.test_216;

fn test_216() {
    let result = 96 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_216()
}
