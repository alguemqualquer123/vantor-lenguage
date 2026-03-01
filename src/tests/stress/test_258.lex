module tests.test_258;

fn test_258() {
    let val: String? = null
    let result = val ?? "default_258"
    assert(result == "default_258")
}


pub fn main() {
    test_258()
}
