module tests.test_160;

fn test_160() {
    let result = 79 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_160()
}
