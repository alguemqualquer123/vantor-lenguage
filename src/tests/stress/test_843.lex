module tests.test_843;

fn test_843() {
    let val: String? = null
    let result = val ?? "default_843"
    assert(result == "default_843")
}


pub fn main() {
    test_843()
}
