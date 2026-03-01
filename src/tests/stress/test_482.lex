module tests.test_482;

fn test_482() {
    let result = 40 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_482()
}
