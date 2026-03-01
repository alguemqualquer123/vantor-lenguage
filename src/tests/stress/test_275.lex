module tests.test_275;

fn test_275() {
    let val: String? = null
    let result = val ?? "default_275"
    assert(result == "default_275")
}


pub fn main() {
    test_275()
}
