module tests.test_889;

fn test_889() {
    let val: String? = null
    let result = val ?? "default_889"
    assert(result == "default_889")
}


pub fn main() {
    test_889()
}
