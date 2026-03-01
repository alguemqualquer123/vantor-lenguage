module tests.test_521;

fn test_521() {
    let val: String? = null
    let result = val ?? "default_521"
    assert(result == "default_521")
}


pub fn main() {
    test_521()
}
