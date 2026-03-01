module tests.test_939;

fn test_939() {
    let val: String? = null
    let result = val ?? "default_939"
    assert(result == "default_939")
}


pub fn main() {
    test_939()
}
