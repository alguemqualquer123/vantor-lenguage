module tests.test_366;

fn test_366() {
    let val: String? = null
    let result = val ?? "default_366"
    assert(result == "default_366")
}


pub fn main() {
    test_366()
}
