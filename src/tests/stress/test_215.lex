module tests.test_215;

fn test_215() {
    let result = 39 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_215()
}
