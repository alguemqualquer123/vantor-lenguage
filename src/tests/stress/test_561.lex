module tests.test_561;

fn test_561() {
    let val: String? = null
    let result = val ?? "default_561"
    assert(result == "default_561")
}


pub fn main() {
    test_561()
}
