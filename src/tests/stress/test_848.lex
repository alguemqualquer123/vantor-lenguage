module tests.test_848;

fn test_848() {
    let result = 7 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_848()
}
