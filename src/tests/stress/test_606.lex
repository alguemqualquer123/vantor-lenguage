module tests.test_606;

fn test_606() {
    let val: String? = null
    let result = val ?? "default_606"
    assert(result == "default_606")
}


pub fn main() {
    test_606()
}
