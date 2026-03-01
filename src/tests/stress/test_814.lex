module tests.test_814;

fn test_814() {
    let val: String? = null
    let result = val ?? "default_814"
    assert(result == "default_814")
}


pub fn main() {
    test_814()
}
