module tests.test_965;

fn test_965() {
    let val: String? = null
    let result = val ?? "default_965"
    assert(result == "default_965")
}


pub fn main() {
    test_965()
}
