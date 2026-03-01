module tests.test_225;

fn test_225() {
    let val: String? = null
    let result = val ?? "default_225"
    assert(result == "default_225")
}


pub fn main() {
    test_225()
}
