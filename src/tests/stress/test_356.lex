module tests.test_356;

fn test_356() {
    let val: String? = null
    let result = val ?? "default_356"
    assert(result == "default_356")
}


pub fn main() {
    test_356()
}
