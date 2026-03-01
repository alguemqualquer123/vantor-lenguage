module tests.test_774;

fn test_774() {
    let result = 88 ▷ |v| v + 5 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_774()
}
