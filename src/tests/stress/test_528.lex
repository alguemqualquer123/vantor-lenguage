module tests.test_528;

fn test_528() {
    let result = 27 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_528()
}
