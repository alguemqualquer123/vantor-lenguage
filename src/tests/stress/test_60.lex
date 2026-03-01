module tests.test_60;

fn test_60() {
    let result = 24 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_60()
}
