module tests.test_174;

fn test_174() {
    let val: String? = null
    let result = val ?? "default_174"
    assert(result == "default_174")
}


pub fn main() {
    test_174()
}
