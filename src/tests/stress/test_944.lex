module tests.test_944;

fn test_944() {
    let val: String? = null
    let result = val ?? "default_944"
    assert(result == "default_944")
}


pub fn main() {
    test_944()
}
