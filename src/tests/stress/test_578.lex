module tests.test_578;

fn test_578() {
    let result = 39 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_578()
}
