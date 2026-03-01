module tests.test_129;

fn test_129() {
    let val: String? = null
    let result = val ?? "default_129"
    assert(result == "default_129")
}


pub fn main() {
    test_129()
}
