module tests.test_393;

fn test_393() {
    let result = 38 ▷ |v| v + 1 ▷ |v| v * 2
    assert(result > 0)
}


pub fn main() {
    test_393()
}
