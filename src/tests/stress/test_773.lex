module tests.test_773;

fn test_773() {
    let val: String? = null
    let result = val ?? "default_773"
    assert(result == "default_773")
}


pub fn main() {
    test_773()
}
