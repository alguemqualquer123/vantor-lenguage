module tests.test_66;

fn test_66() {
    let val: String? = null
    let result = val ?? "default_66"
    assert(result == "default_66")
}


pub fn main() {
    test_66()
}
