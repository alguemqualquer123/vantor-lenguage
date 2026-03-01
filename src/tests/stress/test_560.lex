module tests.test_560;

fn test_560() {
    let val: String? = null
    let result = val ?? "default_560"
    assert(result == "default_560")
}


pub fn main() {
    test_560()
}
