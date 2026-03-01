module tests.test_930;

fn test_930() {
    let val: String? = null
    let result = val ?? "default_930"
    assert(result == "default_930")
}


pub fn main() {
    test_930()
}
