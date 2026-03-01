module tests.test_784;

fn test_784() {
    let result = 8 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_784()
}
