module tests.test_904;

fn test_904() {
    let result = 58 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_904()
}
