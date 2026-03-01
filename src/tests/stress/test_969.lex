module tests.test_969;

fn test_969() {
    let result = 91 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_969()
}
