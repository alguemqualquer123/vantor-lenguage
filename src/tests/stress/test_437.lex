module tests.test_437;

fn test_437() {
    let result = 18 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_437()
}
