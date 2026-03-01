module tests.test_465;

fn test_465() {
    let result = 47 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_465()
}
