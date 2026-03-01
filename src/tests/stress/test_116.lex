module tests.test_116;

fn test_116() {
    let val: String? = null
    let result = val ?? "default_116"
    assert(result == "default_116")
}


pub fn main() {
    test_116()
}
