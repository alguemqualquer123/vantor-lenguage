module tests.test_118;

fn test_118() {
    let val: String? = null
    let result = val ?? "default_118"
    assert(result == "default_118")
}


pub fn main() {
    test_118()
}
