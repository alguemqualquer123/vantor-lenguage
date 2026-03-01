module tests.test_156;

fn test_156() {
    let result = 40 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_156()
}
