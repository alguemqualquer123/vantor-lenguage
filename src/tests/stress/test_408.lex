module tests.test_408;

fn test_408() {
    let val: String? = null
    let result = val ?? "default_408"
    assert(result == "default_408")
}


pub fn main() {
    test_408()
}
