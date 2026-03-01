module tests.test_781;

fn test_781() {
    let result = 20 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_781()
}
