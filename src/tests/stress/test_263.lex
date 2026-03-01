module tests.test_263;

fn test_263() {
    let val: String? = null
    let result = val ?? "default_263"
    assert(result == "default_263")
}


pub fn main() {
    test_263()
}
