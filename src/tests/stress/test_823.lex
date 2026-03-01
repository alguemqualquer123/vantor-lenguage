module tests.test_823;

fn test_823() {
    let result = 71 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_823()
}
