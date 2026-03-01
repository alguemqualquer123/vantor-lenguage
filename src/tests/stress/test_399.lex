module tests.test_399;

fn test_399() {
    let val: String? = null
    let result = val ?? "default_399"
    assert(result == "default_399")
}


pub fn main() {
    test_399()
}
