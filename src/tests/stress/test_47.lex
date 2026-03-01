module tests.test_47;

fn test_47() {
    let result = 37 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_47()
}
