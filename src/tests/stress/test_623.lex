module tests.test_623;

fn test_623() {
    let val: String? = null
    let result = val ?? "default_623"
    assert(result == "default_623")
}


pub fn main() {
    test_623()
}
