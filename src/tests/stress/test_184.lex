module tests.test_184;

fn test_184() {
    let val: String? = null
    let result = val ?? "default_184"
    assert(result == "default_184")
}


pub fn main() {
    test_184()
}
