module tests.test_331;

fn test_331() {
    let result = 39 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_331()
}
