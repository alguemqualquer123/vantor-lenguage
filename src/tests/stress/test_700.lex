module tests.test_700;

fn test_700() {
    let val: String? = null
    let result = val ?? "default_700"
    assert(result == "default_700")
}


pub fn main() {
    test_700()
}
