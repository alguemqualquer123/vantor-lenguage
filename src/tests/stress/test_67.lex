module tests.test_67;

fn test_67() {
    let val: String? = null
    let result = val ?? "default_67"
    assert(result == "default_67")
}


pub fn main() {
    test_67()
}
