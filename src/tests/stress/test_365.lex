module tests.test_365;

fn test_365() {
    let val: String? = null
    let result = val ?? "default_365"
    assert(result == "default_365")
}


pub fn main() {
    test_365()
}
