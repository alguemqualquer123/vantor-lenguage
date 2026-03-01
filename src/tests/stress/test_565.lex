module tests.test_565;

fn test_565() {
    let val: String? = null
    let result = val ?? "default_565"
    assert(result == "default_565")
}


pub fn main() {
    test_565()
}
