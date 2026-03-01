module tests.test_535;

fn test_535() {
    let result = 45 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_535()
}
