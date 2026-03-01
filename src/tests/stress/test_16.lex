module tests.test_16;

fn test_16() {
    let val: String? = null
    let result = val ?? "default_16"
    assert(result == "default_16")
}


pub fn main() {
    test_16()
}
