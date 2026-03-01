module tests.test_692;

fn test_692() {
    let result = 44 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_692()
}
