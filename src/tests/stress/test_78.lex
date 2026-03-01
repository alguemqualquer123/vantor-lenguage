module tests.test_78;

fn test_78() {
    let val: String? = null
    let result = val ?? "default_78"
    assert(result == "default_78")
}


pub fn main() {
    test_78()
}
