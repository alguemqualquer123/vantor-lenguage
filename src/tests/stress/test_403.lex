module tests.test_403;

fn test_403() {
    let val: String? = null
    let result = val ?? "default_403"
    assert(result == "default_403")
}


pub fn main() {
    test_403()
}
