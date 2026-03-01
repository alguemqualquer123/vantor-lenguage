module tests.test_3;

fn test_3() {
    let val: String? = null
    let result = val ?? "default_3"
    assert(result == "default_3")
}


pub fn main() {
    test_3()
}
