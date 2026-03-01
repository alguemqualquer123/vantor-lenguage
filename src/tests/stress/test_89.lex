module tests.test_89;

fn test_89() {
    let val: String? = null
    let result = val ?? "default_89"
    assert(result == "default_89")
}


pub fn main() {
    test_89()
}
