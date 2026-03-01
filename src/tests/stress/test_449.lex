module tests.test_449;

fn test_449() {
    let val: String? = null
    let result = val ?? "default_449"
    assert(result == "default_449")
}


pub fn main() {
    test_449()
}
