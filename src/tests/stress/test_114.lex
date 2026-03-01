module tests.test_114;

fn test_114() {
    let result = 52 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_114()
}
