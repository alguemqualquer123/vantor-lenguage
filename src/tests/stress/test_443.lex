module tests.test_443;

fn test_443() {
    let val: String? = null
    let result = val ?? "default_443"
    assert(result == "default_443")
}


pub fn main() {
    test_443()
}
