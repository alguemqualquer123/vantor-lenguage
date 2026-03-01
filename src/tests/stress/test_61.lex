module tests.test_61;

fn test_61() {
    let val: String? = null
    let result = val ?? "default_61"
    assert(result == "default_61")
}


pub fn main() {
    test_61()
}
