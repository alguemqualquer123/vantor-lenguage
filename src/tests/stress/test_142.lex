module tests.test_142;

fn test_142() {
    let result = 49 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_142()
}
