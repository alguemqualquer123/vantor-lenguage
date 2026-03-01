module tests.test_626;

fn test_626() {
    let result = 53 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_626()
}
