module tests.test_444;

fn test_444() {
    let result = 29 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_444()
}
