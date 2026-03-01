module tests.test_752;

fn test_752() {
    let val: String? = null
    let result = val ?? "default_752"
    assert(result == "default_752")
}


pub fn main() {
    test_752()
}
