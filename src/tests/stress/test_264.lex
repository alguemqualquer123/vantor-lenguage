module tests.test_264;

fn test_264() {
    let val: String? = null
    let result = val ?? "default_264"
    assert(result == "default_264")
}


pub fn main() {
    test_264()
}
