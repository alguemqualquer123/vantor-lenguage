module tests.test_31;

fn test_31() {
    let val: String? = null
    let result = val ?? "default_31"
    assert(result == "default_31")
}


pub fn main() {
    test_31()
}
