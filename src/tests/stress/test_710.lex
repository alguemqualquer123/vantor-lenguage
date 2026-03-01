module tests.test_710;

fn test_710() {
    let result = 75 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_710()
}
