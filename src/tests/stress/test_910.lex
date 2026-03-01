module tests.test_910;

fn test_910() {
    let result = 60 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_910()
}
