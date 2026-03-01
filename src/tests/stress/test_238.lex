module tests.test_238;

fn test_238() {
    let val: String? = null
    let result = val ?? "default_238"
    assert(result == "default_238")
}


pub fn main() {
    test_238()
}
