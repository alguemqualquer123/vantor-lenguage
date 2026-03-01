module tests.test_382;

fn test_382() {
    let result = 5 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_382()
}
