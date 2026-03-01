module tests.test_915;

fn test_915() {
    let result = 22 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_915()
}
