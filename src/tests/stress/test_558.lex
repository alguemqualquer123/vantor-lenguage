module tests.test_558;

fn test_558() {
    let val: String? = null
    let result = val ?? "default_558"
    assert(result == "default_558")
}


pub fn main() {
    test_558()
}
