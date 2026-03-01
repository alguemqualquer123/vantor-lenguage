module tests.test_982;

fn test_982() {
    let result = 91 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_982()
}
