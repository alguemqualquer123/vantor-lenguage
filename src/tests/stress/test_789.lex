module tests.test_789;

fn test_789() {
    let val: String? = null
    let result = val ?? "default_789"
    assert(result == "default_789")
}


pub fn main() {
    test_789()
}
