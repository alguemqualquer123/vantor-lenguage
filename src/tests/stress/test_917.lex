module tests.test_917;

fn test_917() {
    let val: String? = null
    let result = val ?? "default_917"
    assert(result == "default_917")
}


pub fn main() {
    test_917()
}
