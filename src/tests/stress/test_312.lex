module tests.test_312;

fn test_312() {
    let val: String? = null
    let result = val ?? "default_312"
    assert(result == "default_312")
}


pub fn main() {
    test_312()
}
