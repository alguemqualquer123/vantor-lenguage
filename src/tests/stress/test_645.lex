module tests.test_645;

fn test_645() {
    let val: String? = null
    let result = val ?? "default_645"
    assert(result == "default_645")
}


pub fn main() {
    test_645()
}
