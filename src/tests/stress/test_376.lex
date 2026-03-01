module tests.test_376;

fn test_376() {
    let val: String? = null
    let result = val ?? "default_376"
    assert(result == "default_376")
}


pub fn main() {
    test_376()
}
