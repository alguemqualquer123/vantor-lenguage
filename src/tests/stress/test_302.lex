module tests.test_302;

fn test_302() {
    let result = 48 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_302()
}
