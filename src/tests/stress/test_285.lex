module tests.test_285;

fn test_285() {
    let result = 53 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_285()
}
