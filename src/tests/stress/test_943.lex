module tests.test_943;

fn test_943() {
    let result = 4 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_943()
}
