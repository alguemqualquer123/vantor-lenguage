module tests.test_923;

fn test_923() {
    let result = 60 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_923()
}
