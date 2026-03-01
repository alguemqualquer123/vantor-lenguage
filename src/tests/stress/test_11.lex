module tests.test_11;

fn test_11() {
    let val: String? = null
    let result = val ?? "default_11"
    assert(result == "default_11")
}


pub fn main() {
    test_11()
}
