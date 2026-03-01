module tests.test_574;

fn test_574() {
    let val: String? = null
    let result = val ?? "default_574"
    assert(result == "default_574")
}


pub fn main() {
    test_574()
}
