module tests.test_879;

fn test_879() {
    let result = 79 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_879()
}
