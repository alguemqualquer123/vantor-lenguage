module tests.test_721;

fn test_721() {
    let val: String? = null
    let result = val ?? "default_721"
    assert(result == "default_721")
}


pub fn main() {
    test_721()
}
