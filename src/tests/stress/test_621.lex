module tests.test_621;

fn test_621() {
    let result = 56 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_621()
}
