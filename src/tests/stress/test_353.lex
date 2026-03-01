module tests.test_353;

fn test_353() {
    let result = 27 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_353()
}
