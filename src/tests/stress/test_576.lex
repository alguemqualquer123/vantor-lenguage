module tests.test_576;

fn test_576() {
    let val: String? = null
    let result = val ?? "default_576"
    assert(result == "default_576")
}


pub fn main() {
    test_576()
}
