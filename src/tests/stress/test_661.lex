module tests.test_661;

fn test_661() {
    let result = 91 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_661()
}
