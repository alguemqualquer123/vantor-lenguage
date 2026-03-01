module tests.test_206;

fn test_206() {
    let result = 19 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_206()
}
