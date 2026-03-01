module tests.test_341;

fn test_341() {
    let result = 30 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_341()
}
