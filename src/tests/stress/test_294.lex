module tests.test_294;

fn test_294() {
    let val: String? = null
    let result = val ?? "default_294"
    assert(result == "default_294")
}


pub fn main() {
    test_294()
}
