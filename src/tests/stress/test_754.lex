module tests.test_754;

fn test_754() {
    let val: String? = null
    let result = val ?? "default_754"
    assert(result == "default_754")
}


pub fn main() {
    test_754()
}
