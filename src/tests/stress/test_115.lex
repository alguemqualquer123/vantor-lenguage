module tests.test_115;

fn test_115() {
    let val: String? = null
    let result = val ?? "default_115"
    assert(result == "default_115")
}


pub fn main() {
    test_115()
}
