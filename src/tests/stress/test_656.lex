module tests.test_656;

fn test_656() {
    let result = 88 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_656()
}
