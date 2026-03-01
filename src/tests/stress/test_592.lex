module tests.test_592;

fn test_592() {
    let val: String? = null
    let result = val ?? "default_592"
    assert(result == "default_592")
}


pub fn main() {
    test_592()
}
