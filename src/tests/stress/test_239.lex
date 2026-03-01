module tests.test_239;

fn test_239() {
    let val: String? = null
    let result = val ?? "default_239"
    assert(result == "default_239")
}


pub fn main() {
    test_239()
}
