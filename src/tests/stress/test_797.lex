module tests.test_797;

fn test_797() {
    let val: String? = null
    let result = val ?? "default_797"
    assert(result == "default_797")
}


pub fn main() {
    test_797()
}
