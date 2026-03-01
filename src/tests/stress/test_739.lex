module tests.test_739;

fn test_739() {
    let val: String? = null
    let result = val ?? "default_739"
    assert(result == "default_739")
}


pub fn main() {
    test_739()
}
