module tests.test_245;

fn test_245() {
    let val: String? = null
    let result = val ?? "default_245"
    assert(result == "default_245")
}


pub fn main() {
    test_245()
}
