module tests.test_315;

fn test_315() {
    let result = 15 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_315()
}
