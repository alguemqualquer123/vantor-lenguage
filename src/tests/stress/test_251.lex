module tests.test_251;

fn test_251() {
    let result = 35 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_251()
}
