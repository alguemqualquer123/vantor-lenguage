module tests.test_825;

fn test_825() {
    let result = 83 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_825()
}
