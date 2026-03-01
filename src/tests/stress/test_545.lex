module tests.test_545;

fn test_545() {
    let result = 78 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_545()
}
