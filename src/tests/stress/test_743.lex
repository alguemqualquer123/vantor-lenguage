module tests.test_743;

fn test_743() {
    let result = 71 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_743()
}
