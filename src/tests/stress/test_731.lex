module tests.test_731;

fn test_731() {
    let val: String? = null
    let result = val ?? "default_731"
    assert(result == "default_731")
}


pub fn main() {
    test_731()
}
