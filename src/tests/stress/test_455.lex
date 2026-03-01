module tests.test_455;

fn test_455() {
    let val: String? = null
    let result = val ?? "default_455"
    assert(result == "default_455")
}


pub fn main() {
    test_455()
}
