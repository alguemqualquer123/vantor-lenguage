module tests.test_427;

fn test_427() {
    let result = 23 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_427()
}
