module tests.test_772;

fn test_772() {
    let val: String? = null
    let result = val ?? "default_772"
    assert(result == "default_772")
}


pub fn main() {
    test_772()
}
