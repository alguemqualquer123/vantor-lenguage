module tests.test_386;

fn test_386() {
    let result = 30 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_386()
}
