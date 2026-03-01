module tests.test_748;

fn test_748() {
    let result = 49 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_748()
}
