module tests.test_811;

fn test_811() {
    let val: String? = null
    let result = val ?? "default_811"
    assert(result == "default_811")
}


pub fn main() {
    test_811()
}
