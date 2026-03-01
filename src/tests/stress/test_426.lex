module tests.test_426;

fn test_426() {
    let result = 6 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_426()
}
