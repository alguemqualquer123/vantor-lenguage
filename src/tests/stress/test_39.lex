module tests.test_39;

fn test_39() {
    let val: String? = null
    let result = val ?? "default_39"
    assert(result == "default_39")
}


pub fn main() {
    test_39()
}
