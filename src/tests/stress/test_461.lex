module tests.test_461;

fn test_461() {
    let result = 76 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_461()
}
