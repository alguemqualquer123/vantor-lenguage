module tests.test_123;

fn test_123() {
    let val: String? = null
    let result = val ?? "default_123"
    assert(result == "default_123")
}


pub fn main() {
    test_123()
}
