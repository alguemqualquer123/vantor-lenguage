module tests.test_327;

fn test_327() {
    let result = 48 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_327()
}
