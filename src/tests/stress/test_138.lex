module tests.test_138;

fn test_138() {
    let result = 85 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_138()
}
