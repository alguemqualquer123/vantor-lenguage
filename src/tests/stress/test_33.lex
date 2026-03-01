module tests.test_33;

fn test_33() {
    let val: String? = null
    let result = val ?? "default_33"
    assert(result == "default_33")
}


pub fn main() {
    test_33()
}
