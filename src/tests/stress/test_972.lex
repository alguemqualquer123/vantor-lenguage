module tests.test_972;

fn test_972() {
    let val: String? = null
    let result = val ?? "default_972"
    assert(result == "default_972")
}


pub fn main() {
    test_972()
}
