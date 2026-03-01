module tests.test_307;

fn test_307() {
    let val: String? = null
    let result = val ?? "default_307"
    assert(result == "default_307")
}


pub fn main() {
    test_307()
}
