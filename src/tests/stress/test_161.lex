module tests.test_161;

fn test_161() {
    let val: String? = null
    let result = val ?? "default_161"
    assert(result == "default_161")
}


pub fn main() {
    test_161()
}
