module tests.test_832;

fn test_832() {
    let result = 21 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_832()
}
