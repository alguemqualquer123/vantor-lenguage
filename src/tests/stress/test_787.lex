module tests.test_787;

fn test_787() {
    let result = 54 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_787()
}
