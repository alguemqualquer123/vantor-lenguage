module tests.test_670;

fn test_670() {
    let result = 18 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_670()
}
