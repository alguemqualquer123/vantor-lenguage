module tests.test_23;

fn test_23() {
    let val: String? = null
    let result = val ?? "default_23"
    assert(result == "default_23")
}


pub fn main() {
    test_23()
}
