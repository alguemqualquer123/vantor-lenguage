module tests.test_830;

fn test_830() {
    let result = 10 ▷ |v| v + 3 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_830()
}
