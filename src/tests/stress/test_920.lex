module tests.test_920;

fn test_920() {
    let val: String? = null
    let result = val ?? "default_920"
    assert(result == "default_920")
}


pub fn main() {
    test_920()
}
