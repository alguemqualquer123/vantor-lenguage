module tests.test_837;

fn test_837() {
    let result = 68 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_837()
}
