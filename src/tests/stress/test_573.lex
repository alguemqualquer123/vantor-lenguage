module tests.test_573;

fn test_573() {
    let val: String? = null
    let result = val ?? "default_573"
    assert(result == "default_573")
}


pub fn main() {
    test_573()
}
