module tests.test_995;

fn test_995() {
    let result = 87 ▷ |v| v + 8 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_995()
}
