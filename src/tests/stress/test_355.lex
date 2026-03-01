module tests.test_355;

fn test_355() {
    let val: String? = null
    let result = val ?? "default_355"
    assert(result == "default_355")
}


pub fn main() {
    test_355()
}
