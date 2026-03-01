module tests.test_802;

fn test_802() {
    let result = 71 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_802()
}
