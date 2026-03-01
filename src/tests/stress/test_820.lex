module tests.test_820;

fn test_820() {
    let val: String? = null
    let result = val ?? "default_820"
    assert(result == "default_820")
}


pub fn main() {
    test_820()
}
