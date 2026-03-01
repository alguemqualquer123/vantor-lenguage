module tests.test_483;

fn test_483() {
    let val: String? = null
    let result = val ?? "default_483"
    assert(result == "default_483")
}


pub fn main() {
    test_483()
}
