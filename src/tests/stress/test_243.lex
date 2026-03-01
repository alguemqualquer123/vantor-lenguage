module tests.test_243;

fn test_243() {
    let val: String? = null
    let result = val ?? "default_243"
    assert(result == "default_243")
}


pub fn main() {
    test_243()
}
