module tests.test_304;

fn test_304() {
    let result = 50 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_304()
}
