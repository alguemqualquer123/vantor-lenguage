module tests.test_204;

fn test_204() {
    let result = 92 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_204()
}
