module tests.test_782;

fn test_782() {
    let val: String? = null
    let result = val ?? "default_782"
    assert(result == "default_782")
}


pub fn main() {
    test_782()
}
