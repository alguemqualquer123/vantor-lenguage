module tests.test_344;

fn test_344() {
    let val: String? = null
    let result = val ?? "default_344"
    assert(result == "default_344")
}


pub fn main() {
    test_344()
}
