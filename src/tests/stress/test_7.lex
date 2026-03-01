module tests.test_7;

fn test_7() {
    let val: String? = null
    let result = val ?? "default_7"
    assert(result == "default_7")
}


pub fn main() {
    test_7()
}
