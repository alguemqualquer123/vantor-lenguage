module tests.test_188;

fn test_188() {
    let result = 4 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_188()
}
