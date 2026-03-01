module tests.test_756;

fn test_756() {
    let result = 76 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_756()
}
