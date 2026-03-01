module tests.test_97;

fn test_97() {
    let result = 81 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_97()
}
