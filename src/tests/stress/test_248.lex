module tests.test_248;

fn test_248() {
    let val: String? = null
    let result = val ?? "default_248"
    assert(result == "default_248")
}


pub fn main() {
    test_248()
}
