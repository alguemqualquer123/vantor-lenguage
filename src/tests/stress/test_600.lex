module tests.test_600;

fn test_600() {
    let result = 34 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_600()
}
