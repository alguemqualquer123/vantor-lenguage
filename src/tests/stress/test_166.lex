module tests.test_166;

fn test_166() {
    let val: String? = null
    let result = val ?? "default_166"
    assert(result == "default_166")
}


pub fn main() {
    test_166()
}
