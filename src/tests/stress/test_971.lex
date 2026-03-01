module tests.test_971;

fn test_971() {
    let val: String? = null
    let result = val ?? "default_971"
    assert(result == "default_971")
}


pub fn main() {
    test_971()
}
