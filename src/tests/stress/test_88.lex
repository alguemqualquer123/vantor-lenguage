module tests.test_88;

fn test_88() {
    let val: String? = null
    let result = val ?? "default_88"
    assert(result == "default_88")
}


pub fn main() {
    test_88()
}
