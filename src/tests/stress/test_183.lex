module tests.test_183;

fn test_183() {
    let result = 51 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_183()
}
