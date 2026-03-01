module tests.test_453;

fn test_453() {
    let val: String? = null
    let result = val ?? "default_453"
    assert(result == "default_453")
}


pub fn main() {
    test_453()
}
