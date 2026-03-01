module tests.test_674;

fn test_674() {
    let result = 56 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_674()
}
