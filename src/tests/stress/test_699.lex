module tests.test_699;

fn test_699() {
    let val: String? = null
    let result = val ?? "default_699"
    assert(result == "default_699")
}


pub fn main() {
    test_699()
}
