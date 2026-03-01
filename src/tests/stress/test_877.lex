module tests.test_877;

fn test_877() {
    let result = 20 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_877()
}
