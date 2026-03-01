module tests.test_669;

fn test_669() {
    let result = 2 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_669()
}
