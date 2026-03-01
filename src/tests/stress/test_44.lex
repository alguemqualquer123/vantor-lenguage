module tests.test_44;

fn test_44() {
    let result = 100 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_44()
}
