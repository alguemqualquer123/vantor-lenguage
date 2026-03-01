module tests.test_165;

fn test_165() {
    let val: String? = null
    let result = val ?? "default_165"
    assert(result == "default_165")
}


pub fn main() {
    test_165()
}
