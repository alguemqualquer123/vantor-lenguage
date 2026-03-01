module tests.test_72;

fn test_72() {
    let val: String? = null
    let result = val ?? "default_72"
    assert(result == "default_72")
}


pub fn main() {
    test_72()
}
