module tests.test_102;

fn test_102() {
    let val: String? = null
    let result = val ?? "default_102"
    assert(result == "default_102")
}


pub fn main() {
    test_102()
}
