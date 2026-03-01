module tests.test_813;

fn test_813() {
    let result = 7 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_813()
}
