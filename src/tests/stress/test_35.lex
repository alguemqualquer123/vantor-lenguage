module tests.test_35;

fn test_35() {
    let result = 84 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_35()
}
