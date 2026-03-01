module tests.test_869;

fn test_869() {
    let result = 9 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_869()
}
