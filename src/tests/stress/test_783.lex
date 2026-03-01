module tests.test_783;

fn test_783() {
    let result = 59 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_783()
}
