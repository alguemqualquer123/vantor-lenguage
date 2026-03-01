module tests.test_64;

fn test_64() {
    let val: String? = null
    let result = val ?? "default_64"
    assert(result == "default_64")
}


pub fn main() {
    test_64()
}
