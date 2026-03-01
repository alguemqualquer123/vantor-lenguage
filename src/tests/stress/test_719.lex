module tests.test_719;

fn test_719() {
    let val: String? = null
    let result = val ?? "default_719"
    assert(result == "default_719")
}


pub fn main() {
    test_719()
}
