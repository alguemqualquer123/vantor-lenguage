module tests.test_696;

fn test_696() {
    let val: String? = null
    let result = val ?? "default_696"
    assert(result == "default_696")
}


pub fn main() {
    test_696()
}
