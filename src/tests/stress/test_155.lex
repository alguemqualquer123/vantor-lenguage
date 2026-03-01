module tests.test_155;

fn test_155() {
    let result = 70 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_155()
}
