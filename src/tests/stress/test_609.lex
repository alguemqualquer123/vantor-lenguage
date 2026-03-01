module tests.test_609;

fn test_609() {
    let result = 84 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_609()
}
