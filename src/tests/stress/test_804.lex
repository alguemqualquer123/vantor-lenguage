module tests.test_804;

fn test_804() {
    let val: String? = null
    let result = val ?? "default_804"
    assert(result == "default_804")
}


pub fn main() {
    test_804()
}
