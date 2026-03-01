module tests.test_850;

fn test_850() {
    let val: String? = null
    let result = val ?? "default_850"
    assert(result == "default_850")
}


pub fn main() {
    test_850()
}
