module tests.test_824;

fn test_824() {
    let result = 90 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_824()
}
