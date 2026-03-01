module tests.test_883;

fn test_883() {
    let val: String? = null
    let result = val ?? "default_883"
    assert(result == "default_883")
}


pub fn main() {
    test_883()
}
