module tests.test_702;

fn test_702() {
    let val: String? = null
    let result = val ?? "default_702"
    assert(result == "default_702")
}


pub fn main() {
    test_702()
}
