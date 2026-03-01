module tests.test_108;

fn test_108() {
    let val: String? = null
    let result = val ?? "default_108"
    assert(result == "default_108")
}


pub fn main() {
    test_108()
}
