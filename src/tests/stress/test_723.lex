module tests.test_723;

fn test_723() {
    let val: String? = null
    let result = val ?? "default_723"
    assert(result == "default_723")
}


pub fn main() {
    test_723()
}
