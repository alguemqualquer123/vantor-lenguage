module tests.test_255;

fn test_255() {
    let val: String? = null
    let result = val ?? "default_255"
    assert(result == "default_255")
}


pub fn main() {
    test_255()
}
