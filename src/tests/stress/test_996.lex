module tests.test_996;

fn test_996() {
    let val: String? = null
    let result = val ?? "default_996"
    assert(result == "default_996")
}


pub fn main() {
    test_996()
}
