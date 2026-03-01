module tests.test_937;

fn test_937() {
    let val: String? = null
    let result = val ?? "default_937"
    assert(result == "default_937")
}


pub fn main() {
    test_937()
}
