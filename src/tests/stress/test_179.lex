module tests.test_179;

fn test_179() {
    let val: String? = null
    let result = val ?? "default_179"
    assert(result == "default_179")
}


pub fn main() {
    test_179()
}
