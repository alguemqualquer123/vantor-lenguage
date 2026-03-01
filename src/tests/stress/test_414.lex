module tests.test_414;

fn test_414() {
    let val: String? = null
    let result = val ?? "default_414"
    assert(result == "default_414")
}


pub fn main() {
    test_414()
}
