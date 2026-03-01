module tests.test_335;

fn test_335() {
    let val: String? = null
    let result = val ?? "default_335"
    assert(result == "default_335")
}


pub fn main() {
    test_335()
}
