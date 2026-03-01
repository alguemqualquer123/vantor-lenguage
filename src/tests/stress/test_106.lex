module tests.test_106;

fn test_106() {
    let val: String? = null
    let result = val ?? "default_106"
    assert(result == "default_106")
}


pub fn main() {
    test_106()
}
