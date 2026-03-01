module tests.test_121;

fn test_121() {
    let val: String? = null
    let result = val ?? "default_121"
    assert(result == "default_121")
}


pub fn main() {
    test_121()
}
