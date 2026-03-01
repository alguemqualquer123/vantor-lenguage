module tests.test_424;

fn test_424() {
    let val: String? = null
    let result = val ?? "default_424"
    assert(result == "default_424")
}


pub fn main() {
    test_424()
}
