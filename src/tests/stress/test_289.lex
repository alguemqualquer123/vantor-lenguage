module tests.test_289;

fn test_289() {
    let result = 56 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_289()
}
