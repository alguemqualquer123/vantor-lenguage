module tests.test_193;

fn test_193() {
    let val: String? = null
    let result = val ?? "default_193"
    assert(result == "default_193")
}


pub fn main() {
    test_193()
}
