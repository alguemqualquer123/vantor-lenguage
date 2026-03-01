module tests.test_203;

fn test_203() {
    let val: String? = null
    let result = val ?? "default_203"
    assert(result == "default_203")
}


pub fn main() {
    test_203()
}
