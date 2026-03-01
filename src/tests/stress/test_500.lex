module tests.test_500;

fn test_500() {
    let val: String? = null
    let result = val ?? "default_500"
    assert(result == "default_500")
}


pub fn main() {
    test_500()
}
