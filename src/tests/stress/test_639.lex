module tests.test_639;

fn test_639() {
    let val: String? = null
    let result = val ?? "default_639"
    assert(result == "default_639")
}


pub fn main() {
    test_639()
}
