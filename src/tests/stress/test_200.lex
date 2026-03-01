module tests.test_200;

fn test_200() {
    let result = 62 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_200()
}
