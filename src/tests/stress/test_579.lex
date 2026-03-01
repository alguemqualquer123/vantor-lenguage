module tests.test_579;

fn test_579() {
    let result = 62 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_579()
}
