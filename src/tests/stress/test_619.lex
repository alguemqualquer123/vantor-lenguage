module tests.test_619;

fn test_619() {
    let val: String? = null
    let result = val ?? "default_619"
    assert(result == "default_619")
}


pub fn main() {
    test_619()
}
