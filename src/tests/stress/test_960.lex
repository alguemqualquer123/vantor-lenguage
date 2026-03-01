module tests.test_960;

fn test_960() {
    let result = 75 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_960()
}
