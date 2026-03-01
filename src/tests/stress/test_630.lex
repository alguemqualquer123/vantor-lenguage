module tests.test_630;

fn test_630() {
    let val: String? = null
    let result = val ?? "default_630"
    assert(result == "default_630")
}


pub fn main() {
    test_630()
}
