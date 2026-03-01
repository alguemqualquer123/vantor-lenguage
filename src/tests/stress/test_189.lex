module tests.test_189;

fn test_189() {
    let result = 52 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_189()
}
