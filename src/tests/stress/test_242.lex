module tests.test_242;

fn test_242() {
    let val: String? = null
    let result = val ?? "default_242"
    assert(result == "default_242")
}


pub fn main() {
    test_242()
}
