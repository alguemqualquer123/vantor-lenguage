module tests.test_292;

fn test_292() {
    let result = 53 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_292()
}
