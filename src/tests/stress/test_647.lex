module tests.test_647;

fn test_647() {
    let val: String? = null
    let result = val ?? "default_647"
    assert(result == "default_647")
}


pub fn main() {
    test_647()
}
