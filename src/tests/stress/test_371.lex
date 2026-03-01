module tests.test_371;

fn test_371() {
    let result = 10 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_371()
}
