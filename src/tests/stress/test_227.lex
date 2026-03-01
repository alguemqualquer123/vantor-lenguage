module tests.test_227;

fn test_227() {
    let result = 94 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_227()
}
