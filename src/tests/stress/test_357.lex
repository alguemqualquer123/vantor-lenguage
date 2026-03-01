module tests.test_357;

fn test_357() {
    let result = 25 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_357()
}
