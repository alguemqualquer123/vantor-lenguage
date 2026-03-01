module tests.test_890;

fn test_890() {
    let val: String? = null
    let result = val ?? "default_890"
    assert(result == "default_890")
}


pub fn main() {
    test_890()
}
