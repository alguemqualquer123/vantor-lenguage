module tests.test_518;

fn test_518() {
    let result = 83 ▷ |v| v + 6 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_518()
}
