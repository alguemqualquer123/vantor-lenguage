module tests.test_191;

fn test_191() {
    let val: String? = null
    let result = val ?? "default_191"
    assert(result == "default_191")
}


pub fn main() {
    test_191()
}
