module tests.test_653;

fn test_653() {
    let result = 39 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_653()
}
