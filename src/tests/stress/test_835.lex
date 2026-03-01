module tests.test_835;

fn test_835() {
    let result = 4 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_835()
}
