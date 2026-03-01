module tests.test_649;

fn test_649() {
    let result = 54 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_649()
}
