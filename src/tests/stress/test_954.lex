module tests.test_954;

fn test_954() {
    let val: String? = null
    let result = val ?? "default_954"
    assert(result == "default_954")
}


pub fn main() {
    test_954()
}
