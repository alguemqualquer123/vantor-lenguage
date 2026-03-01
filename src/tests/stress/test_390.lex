module tests.test_390;

fn test_390() {
    let val: String? = null
    let result = val ?? "default_390"
    assert(result == "default_390")
}


pub fn main() {
    test_390()
}
