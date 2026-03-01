module tests.test_153;

fn test_153() {
    let val: String? = null
    let result = val ?? "default_153"
    assert(result == "default_153")
}


pub fn main() {
    test_153()
}
