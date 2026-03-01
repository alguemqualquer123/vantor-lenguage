module tests.test_800;

fn test_800() {
    let result = 94 ▷ |v| v + 2 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_800()
}
