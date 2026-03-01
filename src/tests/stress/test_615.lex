module tests.test_615;

fn test_615() {
    let result = 75 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_615()
}
