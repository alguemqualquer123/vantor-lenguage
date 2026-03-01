module tests.test_136;

fn test_136() {
    let result = 24 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_136()
}
