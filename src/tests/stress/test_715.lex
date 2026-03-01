module tests.test_715;

fn test_715() {
    let val: String? = null
    let result = val ?? "default_715"
    assert(result == "default_715")
}


pub fn main() {
    test_715()
}
