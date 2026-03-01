module tests.test_651;

fn test_651() {
    let val: String? = null
    let result = val ?? "default_651"
    assert(result == "default_651")
}


pub fn main() {
    test_651()
}
