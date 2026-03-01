module tests.test_485;

fn test_485() {
    let val: String? = null
    let result = val ?? "default_485"
    assert(result == "default_485")
}


pub fn main() {
    test_485()
}
