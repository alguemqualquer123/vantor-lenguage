module tests.test_664;

fn test_664() {
    let val: String? = null
    let result = val ?? "default_664"
    assert(result == "default_664")
}


pub fn main() {
    test_664()
}
