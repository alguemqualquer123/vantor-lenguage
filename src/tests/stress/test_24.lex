module tests.test_24;

fn test_24() {
    let result = 3 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_24()
}
