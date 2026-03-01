module tests.test_377;

fn test_377() {
    let val: String? = null
    let result = val ?? "default_377"
    assert(result == "default_377")
}


pub fn main() {
    test_377()
}
