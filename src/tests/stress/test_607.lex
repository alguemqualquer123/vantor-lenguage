module tests.test_607;

fn test_607() {
    let result = 90 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_607()
}
