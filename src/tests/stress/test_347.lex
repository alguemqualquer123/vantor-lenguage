module tests.test_347;

fn test_347() {
    let result = 82 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_347()
}
