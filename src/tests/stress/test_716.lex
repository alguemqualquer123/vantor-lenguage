module tests.test_716;

fn test_716() {
    let result = 89 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_716()
}
