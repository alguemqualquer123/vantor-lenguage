module tests.test_491;

fn test_491() {
    let result = 87 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_491()
}
