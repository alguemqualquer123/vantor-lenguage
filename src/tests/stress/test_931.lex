module tests.test_931;

fn test_931() {
    let val: String? = null
    let result = val ?? "default_931"
    assert(result == "default_931")
}


pub fn main() {
    test_931()
}
