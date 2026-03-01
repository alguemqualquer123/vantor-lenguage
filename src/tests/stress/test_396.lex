module tests.test_396;

fn test_396() {
    let val: String? = null
    let result = val ?? "default_396"
    assert(result == "default_396")
}


pub fn main() {
    test_396()
}
