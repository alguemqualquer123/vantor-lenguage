module tests.test_684;

fn test_684() {
    let result = 84 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_684()
}
