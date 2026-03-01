module tests.test_130;

fn test_130() {
    let result = 22 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_130()
}
