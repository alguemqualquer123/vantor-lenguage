module tests.test_509;

fn test_509() {
    let val: String? = null
    let result = val ?? "default_509"
    assert(result == "default_509")
}


pub fn main() {
    test_509()
}
