module tests.test_290;

fn test_290() {
    let result = 34 ▷ |v| v + 9 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_290()
}
