module tests.test_726;

fn test_726() {
    let result = 59 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_726()
}
