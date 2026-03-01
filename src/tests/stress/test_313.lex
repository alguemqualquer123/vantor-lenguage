module tests.test_313;

fn test_313() {
    let result = 13 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_313()
}
