module tests.test_435;

fn test_435() {
    let val: String? = null
    let result = val ?? "default_435"
    assert(result == "default_435")
}


pub fn main() {
    test_435()
}
