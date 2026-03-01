module tests.test_690;

fn test_690() {
    let result = 73 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_690()
}
