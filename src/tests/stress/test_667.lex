module tests.test_667;

fn test_667() {
    let val: String? = null
    let result = val ?? "default_667"
    assert(result == "default_667")
}


pub fn main() {
    test_667()
}
