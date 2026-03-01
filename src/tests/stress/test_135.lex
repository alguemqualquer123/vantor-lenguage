module tests.test_135;

fn test_135() {
    let val: String? = null
    let result = val ?? "default_135"
    assert(result == "default_135")
}


pub fn main() {
    test_135()
}
