module tests.test_909;

fn test_909() {
    let val: String? = null
    let result = val ?? "default_909"
    assert(result == "default_909")
}


pub fn main() {
    test_909()
}
