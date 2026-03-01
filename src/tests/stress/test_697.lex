module tests.test_697;

fn test_697() {
    let result = 89 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_697()
}
