module tests.test_221;

fn test_221() {
    let result = 42 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_221()
}
