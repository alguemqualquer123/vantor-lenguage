module tests.test_758;

fn test_758() {
    let val: String? = null
    let result = val ?? "default_758"
    assert(result == "default_758")
}


pub fn main() {
    test_758()
}
