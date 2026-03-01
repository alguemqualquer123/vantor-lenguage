module tests.test_267;

fn test_267() {
    let val: String? = null
    let result = val ?? "default_267"
    assert(result == "default_267")
}


pub fn main() {
    test_267()
}
