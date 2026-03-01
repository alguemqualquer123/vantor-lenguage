module tests.test_413;

fn test_413() {
    let val: String? = null
    let result = val ?? "default_413"
    assert(result == "default_413")
}


pub fn main() {
    test_413()
}
