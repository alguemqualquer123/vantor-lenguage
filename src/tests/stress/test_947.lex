module tests.test_947;

fn test_947() {
    let val: String? = null
    let result = val ?? "default_947"
    assert(result == "default_947")
}


pub fn main() {
    test_947()
}
