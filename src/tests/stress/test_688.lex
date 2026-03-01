module tests.test_688;

fn test_688() {
    let val: String? = null
    let result = val ?? "default_688"
    assert(result == "default_688")
}


pub fn main() {
    test_688()
}
