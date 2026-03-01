module tests.test_778;

fn test_778() {
    let val: String? = null
    let result = val ?? "default_778"
    assert(result == "default_778")
}


pub fn main() {
    test_778()
}
