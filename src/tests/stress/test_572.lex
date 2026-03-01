module tests.test_572;

fn test_572() {
    let result = 63 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_572()
}
