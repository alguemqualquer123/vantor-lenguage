module tests.test_729;

fn test_729() {
    let val: String? = null
    let result = val ?? "default_729"
    assert(result == "default_729")
}


pub fn main() {
    test_729()
}
