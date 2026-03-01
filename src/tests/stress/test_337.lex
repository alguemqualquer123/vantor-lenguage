module tests.test_337;

fn test_337() {
    let result = 80 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_337()
}
