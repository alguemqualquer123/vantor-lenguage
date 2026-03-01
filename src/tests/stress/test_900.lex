module tests.test_900;

fn test_900() {
    let result = 74 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_900()
}
