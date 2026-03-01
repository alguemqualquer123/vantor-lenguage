module tests.test_282;

fn test_282() {
    let result = 46 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_282()
}
