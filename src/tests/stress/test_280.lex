module tests.test_280;

fn test_280() {
    let val: String? = null
    let result = val ?? "default_280"
    assert(result == "default_280")
}


pub fn main() {
    test_280()
}
