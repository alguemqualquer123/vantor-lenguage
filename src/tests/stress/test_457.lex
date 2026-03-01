module tests.test_457;

fn test_457() {
    let result = 29 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_457()
}
