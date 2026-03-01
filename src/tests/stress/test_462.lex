module tests.test_462;

fn test_462() {
    let val: String? = null
    let result = val ?? "default_462"
    assert(result == "default_462")
}


pub fn main() {
    test_462()
}
