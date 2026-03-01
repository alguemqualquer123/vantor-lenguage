module tests.test_793;

fn test_793() {
    let result = 29 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_793()
}
