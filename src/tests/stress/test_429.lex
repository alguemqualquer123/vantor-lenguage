module tests.test_429;

fn test_429() {
    let result = 86 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_429()
}
