module tests.test_530;

fn test_530() {
    let val: String? = null
    let result = val ?? "default_530"
    assert(result == "default_530")
}


pub fn main() {
    test_530()
}
