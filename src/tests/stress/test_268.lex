module tests.test_268;

fn test_268() {
    let val: String? = null
    let result = val ?? "default_268"
    assert(result == "default_268")
}


pub fn main() {
    test_268()
}
