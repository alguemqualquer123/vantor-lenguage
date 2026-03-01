module tests.test_925;

fn test_925() {
    let val: String? = null
    let result = val ?? "default_925"
    assert(result == "default_925")
}


pub fn main() {
    test_925()
}
