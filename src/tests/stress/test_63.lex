module tests.test_63;

fn test_63() {
    let result = 24 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_63()
}
