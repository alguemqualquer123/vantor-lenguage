module tests.test_230;

fn test_230() {
    let val: String? = null
    let result = val ?? "default_230"
    assert(result == "default_230")
}


pub fn main() {
    test_230()
}
