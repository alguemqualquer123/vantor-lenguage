module tests.test_908;

fn test_908() {
    let val: String? = null
    let result = val ?? "default_908"
    assert(result == "default_908")
}


pub fn main() {
    test_908()
}
