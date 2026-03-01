module tests.test_240;

fn test_240() {
    let val: String? = null
    let result = val ?? "default_240"
    assert(result == "default_240")
}


pub fn main() {
    test_240()
}
