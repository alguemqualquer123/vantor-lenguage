module tests.test_957;

fn test_957() {
    let result = 98 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_957()
}
