module tests.test_380;

fn test_380() {
    let result = 5 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_380()
}
