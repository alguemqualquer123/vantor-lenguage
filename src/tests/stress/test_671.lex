module tests.test_671;

fn test_671() {
    let result = 89 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_671()
}
