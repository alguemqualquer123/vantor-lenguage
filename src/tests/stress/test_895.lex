module tests.test_895;

fn test_895() {
    let result = 15 ▷ |v| v + 4 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_895()
}
