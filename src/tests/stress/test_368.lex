module tests.test_368;

fn test_368() {
    let val: String? = null
    let result = val ?? "default_368"
    assert(result == "default_368")
}


pub fn main() {
    test_368()
}
