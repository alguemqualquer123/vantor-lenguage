module tests.test_232;

fn test_232() {
    let val: String? = null
    let result = val ?? "default_232"
    assert(result == "default_232")
}


pub fn main() {
    test_232()
}
