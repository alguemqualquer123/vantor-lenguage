module tests.test_539;

fn test_539() {
    let result = 99 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_539()
}
