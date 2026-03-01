module tests.test_417;

fn test_417() {
    let val: String? = null
    let result = val ?? "default_417"
    assert(result == "default_417")
}


pub fn main() {
    test_417()
}
