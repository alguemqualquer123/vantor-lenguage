module tests.test_547;

fn test_547() {
    let val: String? = null
    let result = val ?? "default_547"
    assert(result == "default_547")
}


pub fn main() {
    test_547()
}
