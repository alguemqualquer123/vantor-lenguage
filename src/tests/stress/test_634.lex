module tests.test_634;

fn test_634() {
    let result = 92 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_634()
}
