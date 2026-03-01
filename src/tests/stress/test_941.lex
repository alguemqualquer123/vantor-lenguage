module tests.test_941;

fn test_941() {
    let result = 64 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_941()
}
