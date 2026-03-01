module tests.test_452;

fn test_452() {
    let result = 37 ▷ |v| v + 10 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_452()
}
