module tests.test_929;

fn test_929() {
    let result = 49 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_929()
}
