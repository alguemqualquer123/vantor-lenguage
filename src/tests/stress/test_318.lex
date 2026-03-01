module tests.test_318;

fn test_318() {
    let val: String? = null
    let result = val ?? "default_318"
    assert(result == "default_318")
}


pub fn main() {
    test_318()
}
