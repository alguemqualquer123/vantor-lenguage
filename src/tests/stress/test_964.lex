module tests.test_964;

fn test_964() {
    let val: String? = null
    let result = val ?? "default_964"
    assert(result == "default_964")
}


pub fn main() {
    test_964()
}
