module tests.test_657;

fn test_657() {
    let result = 62 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_657()
}
