module tests.test_749;

fn test_749() {
    let val: String? = null
    let result = val ?? "default_749"
    assert(result == "default_749")
}


pub fn main() {
    test_749()
}
