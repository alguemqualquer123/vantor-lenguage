module tests.test_144;

fn test_144() {
    let result = 23 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_144()
}
