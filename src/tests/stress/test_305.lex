module tests.test_305;

fn test_305() {
    let result = 88 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_305()
}
