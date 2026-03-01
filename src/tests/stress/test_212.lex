module tests.test_212;

fn test_212() {
    let val: String? = null
    let result = val ?? "default_212"
    assert(result == "default_212")
}


pub fn main() {
    test_212()
}
