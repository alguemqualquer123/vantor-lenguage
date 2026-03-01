module tests.test_271;

fn test_271() {
    let val: String? = null
    let result = val ?? "default_271"
    assert(result == "default_271")
}


pub fn main() {
    test_271()
}
