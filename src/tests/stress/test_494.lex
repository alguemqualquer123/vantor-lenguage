module tests.test_494;

fn test_494() {
    let val: String? = null
    let result = val ?? "default_494"
    assert(result == "default_494")
}


pub fn main() {
    test_494()
}
