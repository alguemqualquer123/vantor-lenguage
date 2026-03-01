module tests.test_532;

fn test_532() {
    let val: String? = null
    let result = val ?? "default_532"
    assert(result == "default_532")
}


pub fn main() {
    test_532()
}
