module tests.test_55;

fn test_55() {
    let result = 5 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_55()
}
