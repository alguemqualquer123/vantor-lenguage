module tests.test_234;

fn test_234() {
    let result = 2 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_234()
}
