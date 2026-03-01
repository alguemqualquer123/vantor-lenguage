module tests.test_785;

fn test_785() {
    let result = 60 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_785()
}
