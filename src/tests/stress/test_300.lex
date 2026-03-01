module tests.test_300;

fn test_300() {
    let result = 70 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_300()
}
