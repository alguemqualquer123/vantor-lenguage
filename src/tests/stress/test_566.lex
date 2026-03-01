module tests.test_566;

fn test_566() {
    let result = 28 ▷ |v| v + 7 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_566()
}
