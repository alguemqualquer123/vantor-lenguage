module tests.test_110;

fn test_110() {
    let result = 25 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_110()
}
