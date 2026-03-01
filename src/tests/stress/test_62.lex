module tests.test_62;

fn test_62() {
    let result = 91 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_62()
}
