module tests.test_354;

fn test_354() {
    let result = 69 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_354()
}
