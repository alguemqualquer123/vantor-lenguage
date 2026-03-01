module tests.test_487;

fn test_487() {
    let result = 66 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_487()
}
