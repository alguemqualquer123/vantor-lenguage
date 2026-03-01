module tests.test_109;

fn test_109() {
    let result = 16 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_109()
}
