module tests.test_418;

fn test_418() {
    let result = 4 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_418()
}
