module tests.test_492;

fn test_492() {
    let result = 70 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_492()
}
