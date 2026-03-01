module tests.test_131;

fn test_131() {
    let val: String? = null
    let result = val ?? "default_131"
    assert(result == "default_131")
}


pub fn main() {
    test_131()
}
