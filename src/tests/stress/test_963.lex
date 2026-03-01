module tests.test_963;

fn test_963() {
    let val: String? = null
    let result = val ?? "default_963"
    assert(result == "default_963")
}


pub fn main() {
    test_963()
}
