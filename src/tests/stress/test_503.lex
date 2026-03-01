module tests.test_503;

fn test_503() {
    let val: String? = null
    let result = val ?? "default_503"
    assert(result == "default_503")
}


pub fn main() {
    test_503()
}
