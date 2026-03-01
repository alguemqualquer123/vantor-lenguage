module tests.test_187;

fn test_187() {
    let val: String? = null
    let result = val ?? "default_187"
    assert(result == "default_187")
}


pub fn main() {
    test_187()
}
