module tests.test_43;

fn test_43() {
    let val: String? = null
    let result = val ?? "default_43"
    assert(result == "default_43")
}


pub fn main() {
    test_43()
}
