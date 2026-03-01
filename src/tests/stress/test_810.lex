module tests.test_810;

fn test_810() {
    let val: String? = null
    let result = val ?? "default_810"
    assert(result == "default_810")
}


pub fn main() {
    test_810()
}
