module tests.test_681;

fn test_681() {
    let val: String? = null
    let result = val ?? "default_681"
    assert(result == "default_681")
}


pub fn main() {
    test_681()
}
