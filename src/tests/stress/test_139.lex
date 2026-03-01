module tests.test_139;

fn test_139() {
    let val: String? = null
    let result = val ?? "default_139"
    assert(result == "default_139")
}


pub fn main() {
    test_139()
}
