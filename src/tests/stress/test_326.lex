module tests.test_326;

fn test_326() {
    let result = 47 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_326()
}
