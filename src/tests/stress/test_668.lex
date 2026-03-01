module tests.test_668;

fn test_668() {
    let result = 28 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_668()
}
