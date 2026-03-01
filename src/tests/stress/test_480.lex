module tests.test_480;

fn test_480() {
    let val: String? = null
    let result = val ?? "default_480"
    assert(result == "default_480")
}


pub fn main() {
    test_480()
}
