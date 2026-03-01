module tests.test_198;

fn test_198() {
    let result = 57 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_198()
}
