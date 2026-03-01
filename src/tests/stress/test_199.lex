module tests.test_199;

fn test_199() {
    let val: String? = null
    let result = val ?? "default_199"
    assert(result == "default_199")
}


pub fn main() {
    test_199()
}
