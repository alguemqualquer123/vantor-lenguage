module tests.test_874;

fn test_874() {
    let result = 35 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_874()
}
