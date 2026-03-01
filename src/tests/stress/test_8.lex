module tests.test_8;

fn test_8() {
    let val: String? = null
    let result = val ?? "default_8"
    assert(result == "default_8")
}


pub fn main() {
    test_8()
}
