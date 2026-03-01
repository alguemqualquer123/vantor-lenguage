module tests.test_96;

fn test_96() {
    let result = 64 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_96()
}
