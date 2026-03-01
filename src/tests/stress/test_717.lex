module tests.test_717;

fn test_717() {
    let result = 28 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_717()
}
