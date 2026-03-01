module tests.test_765;

fn test_765() {
    let val: String? = null
    let result = val ?? "default_765"
    assert(result == "default_765")
}


pub fn main() {
    test_765()
}
