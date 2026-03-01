module tests.test_605;

fn test_605() {
    let result = 79 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_605()
}
