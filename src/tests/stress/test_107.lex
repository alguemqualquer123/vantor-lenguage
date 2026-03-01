module tests.test_107;

fn test_107() {
    let result = 29 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_107()
}
