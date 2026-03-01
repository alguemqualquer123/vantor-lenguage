module tests.test_71;

fn test_71() {
    let val: String? = null
    let result = val ?? "default_71"
    assert(result == "default_71")
}


pub fn main() {
    test_71()
}
