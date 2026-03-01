module tests.test_339;

fn test_339() {
    let val: String? = null
    let result = val ?? "default_339"
    assert(result == "default_339")
}


pub fn main() {
    test_339()
}
