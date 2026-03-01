module tests.test_708;

fn test_708() {
    let val: String? = null
    let result = val ?? "default_708"
    assert(result == "default_708")
}


pub fn main() {
    test_708()
}
